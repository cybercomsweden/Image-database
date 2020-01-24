use postgres_types::{FromSql, Kind, ToSql, Type};
use std::error::Error;

use crate::coord::Location;

#[derive(Clone, Debug, PartialEq, FromSql, ToSql)]
#[postgres(name = "entity_type")]
pub enum EntityType {
    #[postgres(name = "image")]
    Image,

    #[postgres(name = "video")]
    Video,
}

impl<'a> FromSql<'a> for Location {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> std::result::Result<Self, Box<dyn Error + 'static + Send + Sync>> {
        Ok(Location::from_postgis_ewkb(&raw)?)
    }

    fn accepts(ty: &Type) -> bool {
        ty.kind() == &Kind::Simple && ty.name() == "geography"
    }
}
