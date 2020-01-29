use anyhow::anyhow;
use futures::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::vec::Vec;
use tokio_postgres::Client;

use crate::error::Result;
use crate::model::{Entity, Tag, TagToEntity};

fn list_children(hm: &HashMap<Option<i32>, Vec<Tag>>, pid: i32, mut tree: Vec<String>) {
    let children = hm.get(&Some(pid));
    if children.is_none() {
        println!("{:?}", tree);
        return;
    }
    for child in children.unwrap() {
        tree.push(child.canonical_name.clone());
        list_children(&hm, child.id, tree.clone());
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

pub async fn search_tag(client: &Client, tag: String) -> Result<HashMap<PathBuf, Vec<Tag>>> {
    // Find tag and all its children
    let mut tids = vec![];
    let mut tags = Box::pin(Tag::search(&client, tag).await?);
    while let Some(tag) = tags.next().await.transpose()? {
        tids.push(tag.id);
    }

    // Find all images with those tags
    let mut eids = vec![];
    for tid in tids {
        let mut entities = Box::pin(TagToEntity::get_from_tid(&client, tid).await?);
        while let Some(entity) = entities.next().await.transpose()? {
            eids.push(entity.eid);
        }
    }

    // Extract paths to images and all tags corresponding to the image
    // (not only from the searched tree)
    let mut imgs = HashMap::new();
    for eid in eids {
        let entity = Box::pin(
            Entity::get(&client, eid)
                .await
                .ok_or(anyhow!("Entity {} not mapped yet", eid))?,
        );
        let mut tags = Box::pin(TagToEntity::get_from_eid(&client, eid).await?);
        // Find all tags or the image
        while let Some(et) = tags.next().await.transpose()? {
            let tag = Tag::get(&client, et.tid)
                .await
                .ok_or(anyhow!("Tag not present"))?;

            imgs.entry(entity.path.clone()).or_insert(vec![]).push(tag);
        }
    }
    Ok(imgs)
}
