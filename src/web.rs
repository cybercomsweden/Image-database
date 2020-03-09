Image database, allows the user to host a database themselves,
with the possibilities to tag and search after images.
Copyright (C) 2020 Cybercom group AB, Sweden
By Christoffer Dahl, Johanna Hultberg, Andreas Runfalk and Margareta Vi

Image database is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_protobuf::ProtoBuf;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::anyhow;
use futures::{Stream, StreamExt, TryStreamExt};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::path::PathBuf;
use std::pin::Pin;

use crate::api;
use crate::config::Config;
use crate::error::Result;
use crate::hash::Sha3;
use crate::metadata::Metadata;
use crate::model::{Entity, Tag, TagToEntity};
use crate::thumbnail::{copy_and_create_thumbnail_bytes, file_type_from_path};
use crate::util::{get_db, get_differences, get_media_type, DbConn};

fn make_protobuf_response<T: prost::Message>(pb: &T) -> Result<impl Responder> {
    let mut buf_mut = Vec::new();
    pb.encode(&mut buf_mut)?;
    Ok(HttpResponse::Ok()
        .content_type("application/protobuf")
        .body(buf_mut))
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

async fn api_media_update(
    db: web::Data<DbConn>,
    ProtoBuf(entity_pb): ProtoBuf<api::Entity>,
) -> Result<impl Responder> {
    // TODO: Use transaction
    let client_ids: Vec<i32> = entity_pb
        .tags
        .unwrap_or(api::Tags::default())
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

    make_protobuf_response(&api::Entity::new_from_db(db_entity, new_tags)?)
}

async fn media_upload(db: web::Data<DbConn>, mut payload: Multipart) -> Result<impl Responder> {
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

async fn api_media_list(
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

async fn api_media_get(req: HttpRequest, db: web::Data<DbConn>) -> Result<impl Responder> {
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

async fn api_tags_list(db: web::Data<DbConn>) -> Result<impl Responder> {
    let mut tags = Box::pin(Tag::list(&db).await?);
    let mut tags_pb = api::Tags::default();
    while let Some(tag) = tags.next().await.transpose()? {
        tags_pb.add(api::Tag::try_from(tag)?);
    }
    make_protobuf_response(&tags_pb)
}

async fn api_tag_get_by_name(req: HttpRequest, db: web::Data<DbConn>) -> Result<impl Responder> {
    let name = Tag::canonical_name(req.match_info().query("name"))?;
    let tag = Box::pin(Tag::get_from_canonical_name(&db, name.clone()))
        .await
        .ok_or(anyhow!("Tag {} not mapped yet", name))?;
    let tag_pb = api::Tag::try_from(tag)?;
    make_protobuf_response(&tag_pb)
}

async fn api_tags_autocomplete(db: web::Data<DbConn>) -> Result<impl Responder> {
    make_protobuf_response(&api::AutocompleteTags::from_db(&db).await?)
}

async fn api_tags_add(
    ProtoBuf(tag_pb): ProtoBuf<api::Tag>,
    db: web::Data<DbConn>,
) -> Result<impl Responder> {
    dbg!(&tag_pb);
    let pid = tag_pb.pid;
    let mut parent = None;
    if pid > 0 {
        let parent_tag = Tag::get(&db, pid)
            .await
            .ok_or(anyhow!("Parent not found"))?;
        parent = Some(parent_tag.canonical_name);
    }
    let tag = Tag::insert(&db, tag_pb.name.as_str(), parent).await?;
    println!("{:#?}", tag);
    let new_tag_pb = api::Tag::try_from(tag)?;

    make_protobuf_response(&new_tag_pb)
}

async fn api_media_delete(
    db: web::Data<DbConn>,
    ProtoBuf(entity_pb): ProtoBuf<api::Entity>,
) -> Result<impl Responder> {
    let id = entity_pb.id;
    Entity::delete(&db, id).await?;
    println!("Deleted image with id: {:?}", id);
    Ok(HttpResponse::Ok())
}

pub async fn run_server(config: Config) -> Result<()> {
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
            .route("/media/upload", web::post().to(media_upload))
            .route("/media/{id}", web::get().to(static_html))
            .route("/media", web::get().to(static_html))
            .route("upload", web::get().to(static_html))
            .route("/assets/{path:.*}", web::get().to(show_media))
            .route("/static/{file}", web::get().to(static_file))
            .route("/api/media", web::get().to(api_media_list))
            .route("/api/media/{id}", web::get().to(api_media_get))
            .route("/api/media/{id}", web::put().to(api_media_update))
            .route("/api/tags", web::get().to(api_tags_list))
            .route(
                "/api/tags/autocomplete",
                web::get().to(api_tags_autocomplete),
            )
            .route("/api/tags/{name}", web::get().to(api_tag_get_by_name))
            .route("/api/media/delete/{id}", web::put().to(api_media_delete))
            .route("/api/tags", web::post().to(api_tags_add))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await?)
}
