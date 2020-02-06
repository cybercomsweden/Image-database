use anyhow::anyhow;
use std::convert::TryFrom;
use std::convert::TryInto;

use crate::model::Entity as DbEntity;
use crate::model::Tag as DbTag;
include!(concat!(env!("OUT_DIR"), "/api.rs"));

impl TryFrom<DbEntity> for Entity {
    type Error = crate::error::Error;
    fn try_from(db_entity: DbEntity) -> crate::error::Result<Entity> {
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
    type Error = crate::error::Error;
    fn try_from(db_tag: DbTag) -> crate::error::Result<Tag> {
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
