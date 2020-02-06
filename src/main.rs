use actix_files::NamedFile;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::anyhow;
use async_std::fs::File as AsyncFile;
use async_std::io::ReadExt;
use futures::{FutureExt, StreamExt};
use prost::Message;
use sha3::digest::Digest;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use tokio_postgres::{Client, Config as PostgresConfig, NoTls};
use walkdir::WalkDir;

mod api;
mod cli;
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
use crate::metadata::{extract_metadata_image_jpg, extract_metadata_video};
use crate::model::{create_schema, Entity, EntityType, Tag};
use crate::tags::{add_parent, list_tags, remove_parent, search_tags, tag_image};
use crate::thumbnail::{copy_and_create_thumbnail, file_type_from_path, FileType, MediaType};

type DbConn = Client;

/// Helper method to access database the database in a request handler. Use by
/// adding `db: web::Data<DbConn>` to your request handler's argument list.
async fn get_db(config: Config) -> Result<DbConn> {
    // TODO: This panics if it's unable to connect to database. How to handle?

    // Create a client that we use to query the database and a connection that
    // we use to wake up the futures when we query the database
    let (client, conn) = PostgresConfig::new()
        .host(&config.database.host)
        .port(config.database.port)
        .user(&config.database.user)
        .dbname(&config.database.dbname)
        .connect(NoTls)
        .await?;

    // We must provide the event loop with our connection, or our query futures
    // will never resolve
    actix_rt::spawn(conn.map(|_| ()));

    Ok(client)
}

async fn show_media(req: HttpRequest) -> Result<NamedFile> {
    // NOTE: Once we have folders here we have to be careful to not introduce security holes
    let path: PathBuf = req.match_info().query("path").parse()?;
    let path = std::path::Path::new("dest").join(path.file_name().ok_or(anyhow!("No such image"))?);
    Ok(NamedFile::open(path)?)
}

async fn static_html() -> Result<NamedFile> {
    Ok(NamedFile::open("src/index.html")?)
}

async fn static_file(req: HttpRequest) -> Result<NamedFile> {
    match req.match_info().query("file") {
        "index.js" => Ok(NamedFile::open("dist/index.js")?),
        "index.js.map" => Ok(NamedFile::open("dist/index.js.map")?),
        "index.css" => Ok(NamedFile::open("dist/index.css")?),
        "index.css.map" => Ok(NamedFile::open("dist/index.css.map")?),
        _ => Err(anyhow!("No such file").into()),
    }
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

async fn get_from_database(req: HttpRequest, db: web::Data<DbConn>) -> Result<impl Responder> {
    let eid = req.match_info().query("id").parse::<i32>().unwrap();
    let entity = Box::pin(Entity::get(&db, eid))
        .await
        .ok_or(anyhow!("Entity {} not mapped yet", eid))?;
    let mut buf_mut = Vec::new();
    let entity_pb = api::Entity::try_from(entity)?;
    entity_pb.encode(&mut buf_mut)?;

    Ok(HttpResponse::Ok()
        .content_type("application/protobuf")
        .body(buf_mut))
}

async fn tags_from_database(db: web::Data<DbConn>) -> Result<impl Responder> {
    let mut tags = Box::pin(Tag::list(&db).await?);
    let mut tags_pb = api::Tags::default();
    while let Some(tag) = tags.next().await.transpose()? {
        tags_pb.add(api::Tag::try_from(tag)?);
    }
    let mut buf_mut = Vec::new();
    tags_pb.encode(&mut buf_mut)?;

    Ok(HttpResponse::Ok()
        .content_type("application/protobuf")
        .body(buf_mut))
}

async fn get_tag_from_database(req: HttpRequest, db: web::Data<DbConn>) -> Result<impl Responder> {
    let name = Tag::canonical_name(req.match_info().query("name"))?;
    let tag = Box::pin(Tag::get_from_canonical_name(&db, name.clone()))
        .await
        .ok_or(anyhow!("Tag {} not mapped yet", name))?;
    let mut buf_mut = Vec::new();
    let tag_pb = api::Tag::try_from(tag)?;
    tag_pb.encode(&mut buf_mut)?;

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
            .wrap(Logger::default())
            .app_data(config.clone())
            .data_factory(move || get_db(get_db_config.clone()))
            .route("/", web::get().to(static_html))
            .route("/tags", web::get().to(static_html))
            .route("/media/{id}", web::get().to(static_html))
            .route("/assets/{path:.*}", web::get().to(show_media))
            .route("/static/{file}", web::get().to(static_file))
            .route("/api/media", web::get().to(list_from_database))
            .route("/api/media/{id}", web::get().to(get_from_database))
            .route("/api/tags", web::get().to(tags_from_database))
            .route("/api/tags/{name}", web::get().to(get_tag_from_database))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await?)
}

async fn sha3_256_file<P: AsRef<Path>>(path: P) -> Result<[u8; 32]> {
    let mut file = AsyncFile::open(path.as_ref().to_owned()).await?;
    let mut buf = [0u8; 4096]; // Use 4096 as the buffer size
    let mut hasher = sha3::Sha3_256::new();
    loop {
        let buf_len = file.read(&mut buf).await?;
        if buf_len == 0 {
            break;
        }
        hasher.input(&buf[..buf_len]);
    }

    Ok(hasher.result().into())
}

async fn populate_database(client: &Client, src_dirs: &Vec<PathBuf>) -> Result<()> {
    for path in src_dirs
        .iter()
        .map(|src_dir| WalkDir::new(src_dir).follow_links(true))
        .flatten()
    {
        let path = path?.into_path();
        if file_type_from_path(&path).is_none() {
            println!("Ignoring {:?}", path);
            continue;
        }

        let metadata = match path.metadata() {
            Ok(m) => m,
            Err(err) => {
                println!("Failed to stat file: {}", err);
                continue;
            }
        };

        // Calculate SHA-3 to see if the file is already imported
        let sha3 = sha3_256_file(&path).await?;
        if let Some(e) = Entity::get_from_sha3(&client, &sha3).await {
            println!("{:?} is already imported (id {})", path, e.id);
            continue;
        }

        println!("Making thumbnail for {:?}", &path);
        let (img, thumbnail, preview) = match copy_and_create_thumbnail(&path) {
            Ok((i, t, p)) => (i, t, p),
            Err(err) => {
                println!("Failed: {}", err);
                continue;
            }
        };

        let entity = Entity::insert(
            &client,
            EntityType::Image,
            &img,
            &thumbnail,
            &preview,
            metadata.len(),
            &sha3,
            &None,
            &None,
        )
        .await?;
        dbg!(entity);
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

    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

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
            let file_type = file_type_from_path(&path).ok_or(anyhow!("Unknown file type"))?;
            if file_type == FileType::Png {
                println!("Cannot show metadata for PNG images");
            } else if file_type == FileType::Jpeg {
                println!("{:#?}", extract_metadata_image_jpg(&path)?);
            } else if file_type.media_type() == MediaType::RawImage {
                println!("Showing metadata for raw images is not supported yet");
            } else if file_type.media_type() == MediaType::Video {
                println!("{:#?}", extract_metadata_video(&path)?);
            }
        }
        Cmd::Search { tags } => {
            dbg!(search_tags(&get_db(config).await?, &tags).await?);
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
        Cmd::Tag(SubCmdTag::AddParent { tag, parent }) => {
            add_parent(&get_db(config).await?, tag, parent).await?;
        }
        Cmd::Tag(SubCmdTag::RemoveParent { tag }) => {
            remove_parent(&get_db(config).await?, tag).await?;
        }
    }
    Ok(())
}
