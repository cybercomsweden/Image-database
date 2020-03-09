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
use bytes::{BufMut, BytesMut};
use postgres_types::{to_sql_checked, FromSql, IsNull, Kind, ToSql, Type};
use std::error::Error as ErrorTrait;

use crate::coord::Location;

#[derive(Clone, Debug, PartialEq, FromSql, ToSql)]
#[postgres(name = "entity_type")]
pub enum EntityType {
    #[postgres(name = "image")]
    Image,

    #[postgres(name = "video")]
    Video,
}

impl ToSql for Location {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> std::result::Result<IsNull, Box<dyn ErrorTrait + 'static + Sync + Send>> {
        // Little endian
        out.put_u8(0x01);

        // Point with SRID
        out.put_u8(0x01);
        out.put_u8(0x00);
        out.put_u8(0x00);
        out.put_u8(0x20);

        // SRID
        out.extend(&4326u32.to_le_bytes());

        out.extend(&self.longitude.to_le_bytes());
        out.extend(&self.latitude.to_le_bytes());

        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.kind() == &Kind::Simple && ty.name() == "geography"
    }

    to_sql_checked!();
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
