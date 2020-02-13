use anyhow::anyhow;
use futures::{pin_mut, StreamExt};
use std::convert::TryFrom;
use std::convert::TryInto;
use tokio_postgres::{Client, Row};

use crate::error::{Error, Result};
use crate::metadata::Metadata as FileMetadata;
use crate::metadata::Rotate as FileRotation;
use crate::metadata::TypeSpecific;
use crate::model::Entity as DbEntity;
use crate::model::EntityType as DbEntityType;
use crate::model::Tag as DbTag;
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

pub fn create_entity_with_metadata(
    db_entity: DbEntity,
    tags: Tags,
) -> crate::error::Result<Entity> {
    let mut pb_entity = Entity::try_from(db_entity)?;
    let path = &pb_entity.path;
    let metadata = FileMetadata::from_file(&path)?;
    if let Some(v) = metadata.date_time {
        pb_entity.created = Some(Timestamp {
            seconds: v.timestamp(),
            nanos: v.timestamp_subsec_nanos().try_into()?,
        });
    }
    let pb_metadata = Metadata::try_from(metadata)?;
    pb_entity.metadata = Some(pb_metadata);
    pb_entity.tags = Some(tags);

    Ok(pb_entity)
}

impl TryFrom<FileMetadata> for Metadata {
    type Error = Error;
    fn try_from(file_metadata: FileMetadata) -> Result<Metadata> {
        let mut metadata = Metadata::default();
        metadata.width = file_metadata.width;
        metadata.height = file_metadata.height;
        if let Some(v) = file_metadata.rotation {
            metadata.rotation = match v {
                FileRotation::Zero => metadata::Rotation::Zero.into(),
                FileRotation::Cw90 => metadata::Rotation::Cw90.into(),
                FileRotation::Ccw90 => metadata::Rotation::Ccw90.into(),
                FileRotation::Cw180 => metadata::Rotation::Cw180.into(),
            };
        }
        match file_metadata.type_specific {
            TypeSpecific::Image(file_img_metadata) => {
                let mut img_metadata = metadata::Image::default();
                if let Some(v) = file_img_metadata.aperture {
                    img_metadata.aperture = v.into()
                }
                if let Some(v) = file_img_metadata.iso {
                    img_metadata.iso = v;
                }
                if let Some(v) = file_img_metadata.flash {
                    img_metadata.flash = v;
                }
                metadata.type_specific = Some(metadata::TypeSpecific::Image(img_metadata));
            }
            TypeSpecific::Video(file_video_metadata) => {
                let mut video_metadata = metadata::Video::default();
                video_metadata.duration = file_video_metadata.duration.into();
                if let Some(v) = file_video_metadata.framerate {
                    video_metadata.frame_rate = v.into();
                }
                metadata.type_specific = Some(metadata::TypeSpecific::Video(video_metadata));
            }
        }

        Ok(metadata)
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
