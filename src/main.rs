use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::anyhow;
use futures::{FutureExt, StreamExt};
use prost::Message;
use std::convert::TryFrom;
use std::path::PathBuf;
use tokio_postgres::{Client, Config as PostgresConfig, NoTls};
use walkdir::WalkDir;

mod cli;
mod api;
mod config;
mod coord;
mod error;
mod face_detection;
mod metadata;
mod model;
mod tags;
mod thumbnail;

use crate::cli::{Args, Cmd, SubCmdTag};
use crate::config::Config;
use crate::error::Result;
use crate::metadata::extract_metadata;
use crate::model::{create_schema, Entity, Tag};
use crate::tags::{list_tags, search_tag, tag_image};
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
    let mut entities = Box::pin(Entity::list_desc(&db).await?);
    let mut entities_pb = api::Entities::default();
    while let Some(entity) = entities.next().await.transpose()? {
        entities_pb.add(api::Entity::try_from(entity)?);
    }
    let mut buf_mut = Vec::new();
    entities_pb.encode(&mut buf_mut)?;

    Ok(HttpResponse::Ok()
        .content_type("application/protobuf")
        .body(buf_mut))
}

async fn run_server(config: Config) -> Result<()> {
    Ok(HttpServer::new(move || {
        // We need this here to ensure ownership for the data_factory callback to move this into
        // itself
        let get_db_config = config.clone();

        App::new()
            .app_data(config.clone())
            .data_factory(move || get_db(get_db_config.clone()))
            .route("/list", web::get().to(list_from_database))
            .route("/media/{media:.*}", web::get().to(show_media))
            .route("/static/stylesheet.css", web::get().to(static_css))
            .route("/", web::get().to(static_html))
            .route("/static/index.js", web::get().to(static_js))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await?)
}

async fn populate_database(client: &Client, src_dirs: &Vec<PathBuf>) -> Result<()> {
    for path in src_dirs
        .iter()
        .map(|src_dir| WalkDir::new(src_dir).follow_links(true))
        .flatten()
    {
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

#[actix_rt::main]
async fn main() -> Result<()> {
    let args = Args::from_args();
    let config = if let Ok(config_str) = std::fs::read_to_string("config.toml") {
        toml::from_str(&config_str)?
    } else {
        Config::default()
    };

    match args.cmd.unwrap_or(Cmd::Run) {
        Cmd::Run => {
            run_server(config).await?;
        }
        Cmd::Import { paths } => {
            populate_database(&get_db(config).await?, &paths).await?;
        }
        Cmd::InitDb => {
            create_schema(&get_db(config).await?).await?;
        }
        Cmd::Metadata { path } => {
            println!("{:#?}", extract_metadata(&path)?);
        }
        Cmd::Search { tag } => {
            println!("{:#?}", search_tag(&get_db(config).await?, tag).await?);
        }
        Cmd::Tag(SubCmdTag::Add {
            name,
            tag_type,
            parent,
        }) => {
            println!(
                "{:#?}",
                Tag::insert(
                    &get_db(config).await?,
                    name.as_str(),
                    tag_type.as_str(),
                    parent,
                )
                .await?
            );
        }
        Cmd::Tag(SubCmdTag::List) => {
            list_tags(&get_db(config).await?).await?;
        }
        Cmd::Tag(SubCmdTag::Image { path, tag }) => {
            tag_image(&get_db(config).await?, &path, tag).await?;
        }
    }
    Ok(())
}
