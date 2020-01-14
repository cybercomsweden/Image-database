use actix_web::{web, App, HttpServer, Responder};
use futures::FutureExt;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio_postgres::{Client, NoTls};

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

async fn greet(db: web::Data<DbConn>) -> Result<impl Responder> {
    let rows = db.query("SELECT * FROM entity", &[]).await?;
    if rows.len() == 0 {
        return Ok("No data in database".into());
    }
    let entity = Entity::from_row(&rows[0])?;
    Ok(format!(
        "SELECT 1 + 1 -> {:?} {}",
        &entity,
        entity.location.as_ref().unwrap()
    ))
}

async fn run_server() -> Result<()> {
    Ok(HttpServer::new(|| {
        App::new()
            .data_factory(get_db)
            .route("/", web::get().to(greet))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await?)
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
            copy_and_create_thumbnail(&path)?;
        }
        Cmd::InitDb => {
            actix_rt::System::new("main")
                .block_on(async move { create_schema(&get_db().await?).await })?;
        }
    }
    Ok(())
}
