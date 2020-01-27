use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::anyhow;
use futures::{FutureExt, StreamExt};
use std::path::PathBuf;
use structopt::StructOpt;
use tokio_postgres::{Client, Config as PostgresConfig, NoTls};
use walkdir::WalkDir;

mod config;
mod coord;
mod error;
mod metadata;
mod model;
mod thumbnail;

use crate::config::Config;
use crate::error::Result;
use crate::metadata::extract_metadata;
use crate::model::{create_schema, Entity, Tag};
use crate::thumbnail::{copy_and_create_thumbnail, media_type_from_path};

type DbConn = Client;

/// Helper method to access database the database in a request handler. Use by
/// adding `db: web::Data<DbConn>` to your request handler's argument list.
async fn get_db(config: Config) -> Result<DbConn> {
    // TODO: This panics if it's unable to connect to database. How to handle?

    // Create a client that we use to query the database and a connection that
    // we use to wake up the futures when we query the database
    let res = PostgresConfig::new()
        .host(&config.database.host)
        .port(config.database.port)
        .user(&config.database.user)
        .dbname(&config.database.dbname)
        .connect(NoTls)
        .await;
    let (client, conn) = res?;

    // We must provide the event loop with our connection, or our query futures
    // will never resolve
    actix_rt::spawn(conn.map(|_| ()));

    Ok(client)
}

async fn show_media(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("media").parse()?;
    let path = std::path::Path::new("dest").join(path.file_name().ok_or(anyhow!("No such image"))?);
    Ok(NamedFile::open(path)?)
}

async fn static_html() -> Result<NamedFile> {
    Ok(NamedFile::open("src/index.html")?)
}

async fn static_js() -> Result<NamedFile> {
    Ok(NamedFile::open("dist/index.js")?)
}

async fn static_css() -> Result<NamedFile> {
    Ok(NamedFile::open("src/stylesheet.css")?)
}

async fn list_from_database(db: web::Data<DbConn>) -> Result<impl Responder> {
    let mut html_thumbnails = Vec::new();
    let mut entities = Box::pin(Entity::list_desc(&db).await?);
    while let Some(entity) = entities.next().await.transpose()? {
        html_thumbnails.push(format!(
            r#"<div class="media-thumbnail"><img src="/media/{}"></div>"#,
            &entity
                .thumbnail_path
                .to_str()
                .ok_or(anyhow!("Invalid character in path"))?
        ));
    }

    let content = format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Backlog</title>
            <link rel="stylesheet" href="/static/stylesheet.css">
        </head>
        <body>
            <div class="content">
                <header>
                    <nav>
                        <a class="active" href="/">Media</a>
                        <a href="/">Tags</a>
                    </nav>
                    <div class="search-bar">
                        <input type="text" name="search">
                    </div>
                </header>
                <div class="media-thumbnail-list">{}</div>
            </div>
         </body>
         </html>
        "#,
        html_thumbnails.join("\n")
    );
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

async fn run_server(config: Config) -> Result<()> {
    Ok(HttpServer::new(move || {
        // We need this here to ensure ownership for the data_factory callback to move this into
        // itself
        let get_db_config = config.clone();

        App::new()
            .app_data(config.clone())
            .data_factory(move || get_db(get_db_config.clone()))
            .route("/", web::get().to(list_from_database))
            .route("/media/{media:.*}", web::get().to(show_media))
            .route("/static/stylesheet.css", web::get().to(static_css))
            .route("/static/index.html", web::get().to(static_html))
            .route("/static/index.js", web::get().to(static_js))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await?)
}

async fn populate_database(client: &Client, src_dir: &PathBuf) -> Result<()> {
    for path in WalkDir::new(src_dir).follow_links(true) {
        let path = path?.into_path();
        if media_type_from_path(&path).is_none() {
            println!("Ignoring {:?}", path);
            continue;
        }

        println!("Making thumbnail for {:?}", &path);
        let (img, thumbnail) = match copy_and_create_thumbnail(&path) {
            Ok((i, t)) => (i, t),
            Err(err) => {
                println!("Failed: {}", err);
                continue;
            }
        };

        client
            .execute(
                "
            INSERT INTO entity(media_type, path, thumbnail_path, preview_path)
            VALUES('image', $1, $2, '')
        ",
                &[
                    &img.to_str()
                        .ok_or(anyhow!("Invalid path to copied original"))?,
                    &thumbnail
                        .to_str()
                        .ok_or(anyhow!("Invalid thumbnail path"))?,
                ],
            )
            .await?;
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
enum Cmd {
    /// Default, starts the application
    Run,

    /// Takes the folder provided and copies it to it pre configured folder with corresponding
    /// thumbnails
    Import {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },

    /// Initialize database
    InitDb,

    /// Get metadata from the image provided
    Metadata {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },

    Tag(SubCmdTag),
}

#[derive(Debug, StructOpt)]
enum SubCmdTag {
    /// Add a new tag to the db, on the format: name type Option(parent)
    Add {
        #[structopt(short = "n", long = "name")]
        name: String,

        #[structopt(short = "t", long = "type")]
        tag_type: String,

        #[structopt(short = "p", long = "parent")]
        parent: Option<i32>,
    },

    /// List all present tags and their relation
    List,
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let config = if let Ok(config_str) = std::fs::read_to_string("config.toml") {
        toml::from_str(&config_str)?
    } else {
        Config::default()
    };

    match opt.cmd.unwrap_or(Cmd::Run) {
        Cmd::Run => {
            run_server(config).await?;
        }
        Cmd::Import { path } => {
            populate_database(&get_db(config).await?, &path).await?;
        }
        Cmd::InitDb => {
            create_schema(&get_db(config).await?).await?;
        }
        Cmd::Metadata { path } => {
            println!("{:#?}", extract_metadata(&path)?);
        }
        Cmd::Tag(SubCmdTag::Add {
            name,
            tag_type,
            parent,
        }) => {
            Tag::insert(
                &get_db(config).await?,
                name.as_str(),
                tag_type.as_str(),
                parent,
            )
            .await?;
        }
        Cmd::Tag(SubCmdTag::List) => {
            println!("TODO List tags");
        }
    }
    Ok(())
}
