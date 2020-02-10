use anyhow::anyhow;
use futures::{pin_mut, StreamExt};
use std::convert::TryFrom;
use std::convert::TryInto;
use tokio_postgres::{Client, Row};

use crate::error::{Error, Result};
use crate::metadata::{extract_metadata_image_jpg, extract_metadata_video};
use crate::model::Entity as DbEntity;
use crate::model::EntityType as DbEntityType;
use crate::model::Tag as DbTag;
use crate::thumbnail::{file_type_from_path, FileType, MediaType};
include!(concat!(env!("OUT_DIR"), "/api.rs"));

impl TryFrom<DbEntity> for Entity {
    type Error = Error;
    fn try_from(db_entity: DbEntity) -> Result<Entity> {
        let mut entity = Entity::default();
        entity.id = db_entity.id.try_into()?;
        let media_type = match db_entity.media_type {
            DbEntityType::Image => entity::EntityType::Image as i32,
            DbEntityType::Video => entity::EntityType::Video as i32,
        };
        entity.media_type = media_type;
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
        if let Some(v) = db_entity.created {
            entity.created = Some(Timestamp {
                seconds: v.timestamp(),
                nanos: v.timestamp_subsec_nanos().try_into()?,
            });
        }
        if let Some(v) = db_entity.location {
            let mut location = entity::Location::default();
            location.latitude = v.latitude;
            location.longitude = v.longitude;
            location.place = v.place;
            entity.location = Some(location);
        }
        Ok(entity)
    }
}

impl Entities {
    pub fn add(&mut self, entity: Entity) {
        self.entity.push(entity);
    }
}

pub fn create_entity_with_metadata(db_entity: DbEntity) -> crate::error::Result<Entity> {
    let mut pb_entity = Entity::try_from(db_entity)?;
    let path = &pb_entity.path;
    let file_type = file_type_from_path(&path).ok_or(anyhow!("Unknown file type"))?;
    let mut pb_metadata = Metadata::default();
    if file_type == FileType::Jpeg {
        let metadata = extract_metadata_image_jpg(&path)?;
        pb_metadata.width = metadata.width;
        pb_metadata.height = metadata.height;
        //TODO: fix exposure_time(also in proto file)
        let mut image_metadata = metadata::Image::default();
        if let Some(v) = metadata.aperture {
            image_metadata.aperture = v.into();
        }
        if let Some(v) = metadata.iso {
            image_metadata.iso = v;
        }
        if let Some(v) = metadata.flash {
            image_metadata.flash = v;
        }
        pb_metadata.type_specific = Some(metadata::TypeSpecific::Image(image_metadata));
    } else if file_type.media_type() == MediaType::Video {
        let metadata = extract_metadata_video(&path)?;
        pb_metadata.width = metadata.width;
        pb_metadata.height = metadata.height;
        let mut video_metadata = metadata::Video::default();
        video_metadata.duration = metadata.duration.into();
        pb_metadata.type_specific = Some(metadata::TypeSpecific::Video(video_metadata));
        //TODO: add rotation and frame rate
    }
    pb_entity.metadata = Some(pb_metadata);

    Ok(pb_entity)
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
