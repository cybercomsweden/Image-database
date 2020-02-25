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
    }
    Ok(())
}
