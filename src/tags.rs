use anyhow::anyhow;
use futures::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::vec::Vec;
use tokio_postgres::Client;

use crate::error::Result;
use crate::model::{Entity, Tag, TagToEntity};

fn list_children(hm: &HashMap<Option<i32>, Vec<Tag>>, pid: i32, tree: Vec<String>) {
    let children = hm.get(&Some(pid));
    if children.is_none() {
        println!("{:?}", tree);
        return;
    }
    for child in children.unwrap() {
        let mut new_tree = tree.clone();
        new_tree.push(child.canonical_name.clone());
        list_children(&hm, child.id, new_tree);
    }
}

pub async fn list_tags(client: &Client) -> Result<()> {
    let mut hm = HashMap::new();
    let mut tags = Box::pin(Tag::list(&client).await?);
    while let Some(tag) = tags.next().await.transpose()? {
        hm.entry(tag.pid).or_insert(vec![]).push(tag);
    }
    for parent in hm.get(&None).ok_or(anyhow!("No tags without parent"))? {
        list_children(&hm, parent.id, vec![parent.canonical_name.clone()]);
    }
    Ok(())
}

pub async fn tag_image(client: &Client, path: &PathBuf, tag: String) -> Result<()> {
    let tag = Tag::get_from_canonical_name(&client, tag)
        .await
        .ok_or(anyhow!("Tag not present"))?;
    let entity = Entity::get_from_path(&client, path.to_str().ok_or(anyhow!("Path not string"))?)
        .await
        .ok_or(anyhow!("Path not present"))?;
    TagToEntity::insert(&client, &tag.id, &entity.id).await?;
    Ok(())
}

pub async fn search_tags(client: &Client, tags: &Vec<String>) -> Result<Vec<Entity>> {
    let mut matches = Box::pin(Tag::search(&client, &tags).await?);
    let mut imgs = vec![];
    while let Some(img) = matches.next().await.transpose()? {
        imgs.push(img);
    }
    Ok(imgs)
}

pub async fn add_parent(client: &Client, child: String, parent: String) -> Result<()> {
    let parent = Tag::get_from_canonical_name(&client, Tag::canonical_name(&parent)?)
        .await
        .ok_or(anyhow!("Parent {} does not exist", parent))?;
    let mut child = Tag::get_from_canonical_name(&client, Tag::canonical_name(&child)?)
        .await
        .ok_or(anyhow!("Child {} does not exist", child))?;
    child.pid = Some(parent.id);
    child.save(&client).await?;
    Ok(())
}

pub async fn remove_parent(client: &Client, tag: String) -> Result<()> {
    let mut tag = Tag::get_from_canonical_name(&client, Tag::canonical_name(&tag)?)
        .await
        .ok_or(anyhow!("Tag {} does not exist", tag))?;
    tag.pid = None;
    tag.save(&client).await?;
    Ok(())
}
