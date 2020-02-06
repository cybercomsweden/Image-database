use anyhow::anyhow;
use futures::{pin_mut, StreamExt};
use std::convert::TryFrom;
use std::convert::TryInto;
use tokio_postgres::{Client, Row};

use crate::error::{Error, Result};
use crate::model::Entity as DbEntity;
use crate::model::Tag as DbTag;
include!(concat!(env!("OUT_DIR"), "/api.rs"));

impl TryFrom<DbEntity> for Entity {
    type Error = Error;
    fn try_from(db_entity: DbEntity) -> Result<Entity> {
        let mut entity = Entity::default();
        entity.id = db_entity.id.try_into()?;
        //TODO: add media type
        //entity.media_type = row.try_get::<_, Entity::EntityType>("media_type");
        entity.path = db_entity
            .path
            .to_str()
            .ok_or(anyhow!("Could not convert path"))?
            .to_string();
        entity.thumbnail_path = db_entity
            .thumbnail_path
            .to_str()
            .ok_or(anyhow!("Could not convert path"))?
            .to_string();
        entity.preview_path = db_entity
            .preview_path
            .to_str()
            .ok_or(anyhow!("Could not convert path"))?
            .to_string();
        let uploaded = db_entity.uploaded;
        entity.uploaded = Some(Timestamp {
            seconds: uploaded.timestamp(),
            nanos: uploaded.timestamp_subsec_nanos().try_into()?,
        });
        //TODO: add created and location
        //let created = db_entity.created;
        //entity.created = Some(Timestamp { seconds: created.timestamp(), nanos: created.timestamp_subsec_nanos().try_into()?});
        //dbg!(db_entity.location);
        Ok(entity)
    }
}

impl Entities {
    pub fn add(&mut self, entity: Entity) {
        self.entity.push(entity);
    }
}

impl TryFrom<DbTag> for Tag {
    type Error = Error;
    fn try_from(db_tag: DbTag) -> Result<Tag> {
        let mut tag = Tag::default();
        tag.id = db_tag.id.try_into()?;
        //tag.pid = db_tag.pid.try_into()?; // TODO: Optional
        tag.canonical_name = db_tag.canonical_name.try_into()?;
        tag.name = db_tag.name.try_into()?;
        Ok(tag)
    }
}

impl Tags {
    pub fn add(&mut self, tag: Tag) {
        self.tag.push(tag);
    }
}

impl AutocompleteTags {
    pub fn from_row(row: &Row) -> Result<AutocompleteTag> {
        Ok(AutocompleteTag {
            canonical_name: row.try_get::<_, String>(0)?,
            path: row.try_get::<_, Vec<String>>(1)?,
        })
    }

    pub async fn from_db(client: &Client) -> Result<Self> {
        let db_tags = client
            .query_raw(
                "
                    WITH RECURSIVE deeptag AS (
                        SELECT id, canonical_name, array[name] AS path FROM tag WHERE pid IS NULL
                        UNION
                        SELECT t.id, t.canonical_name, array_append(dt.path, t.name) FROM tag t JOIN deeptag dt ON dt.id = t.pid
                    )
                    SELECT canonical_name, path FROM deeptag
                ",
                vec![],
            )
            .await?
            .map(|row| -> Result<AutocompleteTag> { Ok(Self::from_row(&row?)?) });
        pin_mut!(db_tags);
        let mut tags = AutocompleteTags::default();
        while let Some(tag) = db_tags.next().await.transpose()? {
            tags.tag.push(tag);
        }
        Ok(tags)
    }
}
