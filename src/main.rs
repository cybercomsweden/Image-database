use actix_web::{web, App, HttpServer, Responder};
use futures::FutureExt;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio_postgres::{Client, NoTls};

mod error;
mod thumbnail;

use crate::error::Result;

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
    let rows = db.query("SELECT 1 + 1", &[]).await?;
    Ok(format!("SELECT 1 + 1 -> {}", rows[0].get::<_, i32>(0)))
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
    /// Takes the folder provided and copies it to it pre configured folder with corresponding
    /// thumbnails
    Import {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    /// Default, starts the application
    Run,
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    match opt.cmd.unwrap_or(Cmd::Run) {
        import @ Cmd::Import { .. } => println!("Importing important stuff! {:?}", import),
        Cmd::Run => {
            println!("Running program");
            actix_rt::System::new("main").block_on(async move { run_server().await })?;
        }
    }
    Ok(())
}
