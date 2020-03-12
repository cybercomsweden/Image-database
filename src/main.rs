/*
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
*/
use std::path::PathBuf;
use tokio_postgres::Client;
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
mod util;
mod web;

use crate::cli::{Args, Cmd, SubCmdTag};
use crate::config::Config;
use crate::error::Result;
use crate::hash::Sha3;
use crate::metadata::Metadata;
use crate::model::{create_schema, Entity, Tag};
use crate::tags::{add_parent, list_tags, remove_parent, search_tags, tag_image};
use crate::thumbnail::{copy_and_create_thumbnail, file_type_from_path};
use crate::util::{get_db, get_media_type};
use crate::web::run_server;

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

async fn delete_image(client: &Client, id: String) -> Result<()> {
    let new_id: i32 = id.parse()?;
    Entity::delete(&client, new_id).await?;
    println!("Deleted image with id: {:?}", new_id);
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
            create_schema(&get_db(config.clone()).await?).await?;
            Tag::insert(&get_db(config.clone()).await?, "Places", None).await?;
            Tag::insert(&get_db(config.clone()).await?, "People", None).await?;
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
        Cmd::Delete { id } => {
            delete_image(&get_db(config).await?, id).await?;
        }
    }
    Ok(())
}
