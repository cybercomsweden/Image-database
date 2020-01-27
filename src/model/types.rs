use anyhow::anyhow;
use postgres_types::{FromSql, Kind, ToSql, Type};
use std::convert::TryFrom;
use std::error::Error as ErrorTrait;

use crate::coord::Location;
use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, FromSql, ToSql)]
#[postgres(name = "entity_type")]
pub enum EntityType {
    #[postgres(name = "image")]
    Image,

    #[postgres(name = "video")]
    Video,
}

#[derive(Clone, Debug, PartialEq, FromSql, ToSql)]
#[postgres(name = "tag_type")]
pub enum TagType {
    #[postgres(name = "person")]
    Person,

    #[postgres(name = "place")]
    Place,

    #[postgres(name = "event")]
    Event,

    #[postgres(name = "other")]
    Other,
}

impl<'a> FromSql<'a> for Location {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> std::result::Result<Self, Box<dyn ErrorTrait + 'static + Send + Sync>> {
        Ok(Location::from_postgis_ewkb(&raw)?)
    }

    fn accepts(ty: &Type) -> bool {
        ty.kind() == &Kind::Simple && ty.name() == "geography"
    }
}

impl TryFrom<&str> for TagType {
    type Error = Error;
    fn try_from(from: &str) -> Result<Self> {
        let from = from.to_lowercase();
        match from.as_str() {
            "person" => Ok(TagType::Person),
            "place" => Ok(TagType::Place),
            "event" => Ok(TagType::Event),
            "other" => Ok(TagType::Other),
            _ => Err(anyhow!("Invalid tag").into()),
        }
    }
}
