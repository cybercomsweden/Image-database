use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::anyhow;
use bytes::BytesMut;
use futures::{FutureExt, Stream, StreamExt, TryStreamExt};
use prost::Message;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio_postgres::{Client, Config as PostgresConfig, NoTls};
use walkdir::WalkDir;

mod api;
mod cli;
mod config;
mod coord;
mod error;
mod face_detection;
mod hash;
mod metadata;
mod model;
mod tags;
mod thumbnail;

use crate::api::{Entity as ApiEntity, Tags as ApiTags};
use crate::cli::{Args, Cmd, SubCmdTag};
use crate::config::Config;
use crate::error::Result;
use crate::hash::Sha3;
use crate::metadata::Metadata;
use crate::model::{create_schema, Entity, EntityType, Tag, TagToEntity};
use crate::tags::{add_parent, list_tags, remove_parent, search_tags, tag_image};
use crate::thumbnail::{
    copy_and_create_thumbnail, copy_and_create_thumbnail_bytes, file_type_from_path, MediaType,
};

type DbConn = Client;

fn get_media_type<P: AsRef<Path>>(path: P) -> Result<EntityType> {
    let file_type = file_type_from_path(path).ok_or(anyhow!("Unknown file type"))?;
    match file_type.media_type() {
        MediaType::Image | MediaType::RawImage => Ok(EntityType::Image),
        MediaType::Video => Ok(EntityType::Video),
    }
}

fn make_protobuf_response<T: prost::Message>(pb: &T) -> Result<impl Responder> {
    let mut buf_mut = Vec::new();
    pb.encode(&mut buf_mut)?;
    Ok(HttpResponse::Ok()
        .content_type("application/protobuf")
        .body(buf_mut))
}

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
        "mapbox-icon.png" => Ok(NamedFile::open("gui/mapbox-icon.png")?),
        _ => Err(anyhow!("No such file").into()),
    }
}

fn get_differences<'a, T, F: Fn(&T) -> U, U: Ord>(
    curr: &'a [T],
    new: &'a [T],
    key: F,
) -> (Vec<&'a T>, Vec<&'a T>) {
    let curr_set: BTreeSet<U> = curr.iter().map(&key).collect();
    let new_set: BTreeSet<U> = new.iter().map(&key).collect();

    let mut to_add = Vec::new();
    for to_add_key in new_set.difference(&curr_set) {
        for v in new {
            if to_add_key == &key(v) {
                to_add.push(v);
            }
        }
    }

    let mut to_remove = Vec::new();
    for to_remove_key in curr_set.difference(&new_set) {
        for v in curr {
            if to_remove_key == &key(v) {
                to_remove.push(v);
            }
        }
    }

    (to_add, to_remove)
}

async fn media_edit(db: web::Data<DbConn>, mut payload: web::Payload) -> Result<impl Responder> {
    // TODO: Use transaction
    let mut body = BytesMut::new();
    while let Some(chunk) = payload
        .next()
        .await
        .transpose()
        .map_err(|e| anyhow!("{}", e))?
    {
        // TODO: Protect from very large payloads
        body.extend_from_slice(&chunk);
    }

    let entity_pb = ApiEntity::decode(body)?;
    let client_ids: Vec<i32> = entity_pb
        .tags
        .unwrap_or(ApiTags::default())
        .tag
        .iter()
        .map(|t| t.id)
        .collect();

    let db_entity = Entity::get(&db, entity_pb.id)
        .await
        .ok_or(anyhow!("No such entity"))?;

    let curr_tags: Vec<Tag> = Tag::get_from_eid(&db, entity_pb.id)
        .await?
        .try_collect()
        .await?;
    let new_tags = Tag::list_from_ids(&db, &client_ids).await?;

    let (tags_to_add, tags_to_remove) = get_differences(&curr_tags, &new_tags, |tag| tag.id);

    for tag in tags_to_add {
        TagToEntity::insert(&db, tag.id, db_entity.id).await?;
    }

    for tag in tags_to_remove {
        TagToEntity::delete(&db, tag.id, db_entity.id).await?;
    }

    make_protobuf_response(&ApiEntity::new_from_db(db_entity, new_tags)?)
}

async fn save_file(db: web::Data<DbConn>, mut payload: Multipart) -> Result<impl Responder> {
    // iterate over multipart stream
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|x| anyhow!("{}", x))?;
        let content_type = field.content_disposition().unwrap();
        if content_type.get_name() != Some("fileToUpload") {
            continue;
        }
        let file_name = content_type.get_filename().unwrap();

        // Field in turn isstream of *Bytes* object
        let mut image_chunks: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            image_chunks.append(&mut data.to_vec());
        }

        if file_type_from_path(&file_name).is_none() {
            println!("Ignoring {:?}", file_name);
            continue;
        }

        let sha3 = Sha3::from_reader(image_chunks.as_slice()).await?;
        if let Some(e) = Entity::get_from_sha3(&db, &sha3).await {
            println!("{:?} is already imported (id {})", file_name, e.id);
            continue;
        }

        println!("Making thumbnail for {:?}", &file_name);
        let (img, thumbnail, preview) =
            match copy_and_create_thumbnail_bytes(file_name, &image_chunks) {
                Ok((i, t, p)) => (i, t, p),
                Err(err) => {
                    println!("Failed: {}", err);
                    continue;
                }
            };

        let path = format!("./dest/{}", &file_name);

        let mut created = None;
        let mut location = None;

        if let Ok(metadata) = Metadata::from_file(&path) {
            created = metadata.date_time;
            location = metadata.gps_location;
        }

        let media_type = get_media_type(&path)?;

        Entity::insert(
            &db,
            media_type,
            &img,
            &thumbnail,
            &preview,
            image_chunks.len().try_into().unwrap(),
            &sha3,
            &created,
            &location,
        )
        .await?;
    }
    Ok(HttpResponse::Ok())
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

async fn list_from_database(
    db: web::Data<DbConn>,
    query_params: web::Query<SearchQuery>,
) -> Result<impl Responder> {
    let mut entities: Pin<Box<dyn Stream<Item = Result<Entity>>>> =
        if let Some(ref query) = query_params.q {
            let tags = query.split(" ").map(|x| x.to_owned()).collect();
            Box::pin(Tag::search(&db, &tags).await?)
        } else {
            Box::pin(Entity::list_desc(&db).await?)
        };

    let mut entities_pb = api::Entities::default();
    while let Some(entity) = entities.next().await.transpose()? {
        entities_pb.add(api::Entity::try_from(entity)?);
    }
    make_protobuf_response(&entities_pb)
}

async fn get_from_database(req: HttpRequest, db: web::Data<DbConn>) -> Result<impl Responder> {
    let eid = req.match_info().query("id").parse::<i32>().unwrap();
    let entity = Box::pin(Entity::get(&db, eid))
        .await
        .ok_or(anyhow!("Entity {} not mapped yet", eid))?;

    let mut tags = Box::pin(Tag::get_from_eid(&db, eid).await?);
    let mut tags_pb = api::Tags::default();
    while let Some(tag) = tags.next().await.transpose()? {
        tags_pb.add(api::Tag::try_from(tag)?);
    }

    let pb_entity = api::create_entity_with_metadata(entity, tags_pb)?;
    make_protobuf_response(&pb_entity)
}

async fn tags_from_database(db: web::Data<DbConn>) -> Result<impl Responder> {
    let mut tags = Box::pin(Tag::list(&db).await?);
    let mut tags_pb = api::Tags::default();
    while let Some(tag) = tags.next().await.transpose()? {
        tags_pb.add(api::Tag::try_from(tag)?);
    }
    make_protobuf_response(&tags_pb)
}

async fn get_tag_from_database(req: HttpRequest, db: web::Data<DbConn>) -> Result<impl Responder> {
    let name = Tag::canonical_name(req.match_info().query("name"))?;
    let tag = Box::pin(Tag::get_from_canonical_name(&db, name.clone()))
        .await
        .ok_or(anyhow!("Tag {} not mapped yet", name))?;
    let tag_pb = api::Tag::try_from(tag)?;
    make_protobuf_response(&tag_pb)
}

async fn autocomplete_tags(db: web::Data<DbConn>) -> Result<impl Responder> {
    make_protobuf_response(&api::AutocompleteTags::from_db(&db).await?)
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
            .route("/map", web::get().to(static_html))
            .route("/media/upload", web::post().to(save_file))
            .route("/media/{id}", web::get().to(static_html))
            .route("/media", web::get().to(static_html))
            .route("upload", web::get().to(static_html))
            .route("/assets/{path:.*}", web::get().to(show_media))
            .route("/static/{file}", web::get().to(static_file))
            .route("/api/media", web::get().to(list_from_database))
            .route("/api/media/{id}", web::get().to(get_from_database))
            .route("/api/media/{id}", web::put().to(media_edit))
            .route("/api/tags", web::get().to(tags_from_database))
            .route("/api/tags/autocomplete", web::get().to(autocomplete_tags))
            .route("/api/tags/{name}", web::get().to(get_tag_from_database))
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
        if file_type_from_path(&path).is_none() {
            println!("Ignoring {:?}", path);
            continue;
        }

        let file_metadata = match path.metadata() {
            Ok(m) => m,
            Err(err) => {
                println!("Failed to stat file: {}", err);
                continue;
            }
        };

        // Calculate SHA-3 to see if the file is already imported
        let sha3 = Sha3::from_path(&path).await?;
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

        let mut created = None;
        let mut location = None;

        if let Ok(metadata) = Metadata::from_file(&path) {
            created = metadata.date_time;
            location = metadata.gps_location;
        }

        let media_type = get_media_type(&path)?;

        let entity = Entity::insert(
            &client,
            media_type,
            &img,
            &thumbnail,
            &preview,
            file_metadata.len(),
            &sha3,
            &created,
            &location,
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
        Cmd::Metadata { path } => match Metadata::from_file(&path) {
            Ok(metadata) => println!("{:#?}", metadata),
            Err(_) => println!(
                "Unable to get metadata for {:?}, maybe it's unsupported",
                &path
            ),
        },
        Cmd::Search { tags } => {
            dbg!(search_tags(&get_db(config).await?, &tags).await?);
        }
        Cmd::Tag(SubCmdTag::Add { name, parent }) => {
            println!(
                "{:#?}",
                Tag::insert(&get_db(config).await?, name.as_str(), parent,).await?
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
