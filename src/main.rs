use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::anyhow;
use futures::FutureExt;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio_postgres::{Client, NoTls};
use walkdir::WalkDir;

mod coord;
mod error;
mod model;
mod thumbnail;

use crate::error::Result;
use crate::model::{create_schema, Entity};
use crate::thumbnail::copy_and_create_thumbnail;

type DbConn = Client;

fn get_db_user() -> Result<String> {
    // TODO: Replace this with something libc-based?
    let output = std::process::Command::new("whoami").output()?;
    Ok(std::str::from_utf8(&output.stdout)?.into())
}

/// Helper method to access database the database in a request handler. Use by
/// adding `db: web::Data<DbConn>` to your request handler's argument list.
async fn get_db() -> Result<DbConn> {
    // Create a client that we use to query the database and a connection that
    // we use to wake up the futures when we query the database
    let (client, conn) = tokio_postgres::connect(
        &format!(
            "host=/var/run/postgresql/ user={} dbname=backlog",
            &get_db_user()?
        ),
        NoTls,
    )
    .await?;

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

async fn list_from_database(db: web::Data<DbConn>) -> Result<impl Responder> {
    let rows = db.query("SELECT * FROM entity", &[]).await?;
    if rows.len() == 0 {
        return Ok("No data in database".into());
    }

    let mut vect = Vec::new();
    for row in rows {
        let entity = Entity::from_row(&row)?;
        vect.push(format!("<img src={:?}>", &entity.thumbnail_path));
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(vect.join("\n")))
}

async fn run_server() -> Result<()> {
    Ok(HttpServer::new(|| {
        App::new()
            .data_factory(get_db)
            .route("/", web::get().to(list_from_database))
            .route("/{media:.*}", web::get().to(show_media))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await?)
}

async fn populate_database(client: &Client, src_dir: &PathBuf) -> Result<()> {
    let valid_extensions = vec!["jpg", "jpeg", "png"];

    for path in WalkDir::new(src_dir).follow_links(true) {
        let path = path?.into_path();

        let extension = path
            .extension()
            .map(std::ffi::OsStr::to_str)
            .flatten()
            .unwrap_or("");
        if !valid_extensions.contains(&extension) {
            continue;
        }

        let (img, thumbnail) = copy_and_create_thumbnail(&path)?;

        client
            .execute(
                "
            INSERT INTO entity(media_type, path, thumbnail_path, preview_path)
            VALUES('image', $1, $2, '')
        ",
                &[
                    &img.to_str().ok_or(anyhow!("Invalid img path"))?,
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
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt.cmd.unwrap_or(Cmd::Run) {
        Cmd::Run => {
            actix_rt::System::new("main").block_on(async move { run_server().await })?;
        }
        Cmd::Import { path } => {
            actix_rt::System::new("main")
                .block_on(async move { populate_database(&get_db().await?, &path).await })?;
        }
        Cmd::InitDb => {
            actix_rt::System::new("main")
                .block_on(async move { create_schema(&get_db().await?).await })?;
        }
    }
    Ok(())
}
