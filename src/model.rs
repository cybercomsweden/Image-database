use chrono::{DateTime, Utc};
use postgres_types::{FromSql, Kind, ToSql, Type};
use std::convert::TryInto;
use std::error::Error;
use std::path::{Path, PathBuf};
use tokio_postgres::{Client, Row};

use crate::error::Result;

#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "entity_type")]
pub enum EntityType {
    #[postgres(name = "image")]
    Image,

    #[postgres(name = "video")]
    Video,
}

#[derive(Debug)]
pub struct Location {
    longitude: f64,
    latitude: f64,
}

#[derive(Debug)]
pub struct Entity {
    id: usize,
    media_type: EntityType,
    path: PathBuf,
    thumbnail_path: PathBuf,
    preview_path: PathBuf,
    uploaded: DateTime<Utc>,
    created: Option<DateTime<Utc>>,
    location: Option<Location>,
}

impl<'a> FromSql<'a> for Location {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> std::result::Result<Self, Box<dyn Error + 'static + Send + Sync>> {
        // TODO: Parse this WKB with SRID
        dbg!(ty);
        dbg!(raw);
        Ok(Location {
            longitude: 0.0,
            latitude: 0.0,
        })
    }

    fn accepts(ty: &Type) -> bool {
        ty.kind() == &Kind::Simple && ty.name() == "geography"
    }
}

impl Entity {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get::<_, i32>("id")?.try_into()?,
            media_type: row.try_get::<_, EntityType>("media_type")?,
            path: Path::new(row.try_get::<_, &str>("path")?).to_path_buf(),
            thumbnail_path: Path::new(row.try_get::<_, &str>("thumbnail_path")?).to_path_buf(),
            preview_path: Path::new(row.try_get::<_, &str>("preview_path")?).to_path_buf(),
            uploaded: row.try_get::<_, DateTime<Utc>>("uploaded")?,
            created: row.try_get::<_, Option<DateTime<Utc>>>("created")?,
            location: row.try_get::<_, Option<Location>>("location")?,
        })
    }
}

pub async fn create_schema(client: &Client) -> Result<()> {
    // TODO: Transaction hanlding?
    client
        .execute("CREATE EXTENSION IF NOT EXISTS postgis", &[])
        .await
        .unwrap();
    client
        .execute(
            "
        DO $$ BEGIN
            CREATE TYPE entity_type AS ENUM ('image', 'video');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$
    ",
            &[],
        )
        .await
        .unwrap();
    client
        .execute(
            "
        CREATE TABLE IF NOT EXISTS entity(
            id serial PRIMARY KEY NOT NULL,
            media_type entity_type NOT NULL,
            path varchar NOT NULL,
            thumbnail_path varchar NOT NULL,
            preview_path varchar NOT NULL,
            uploaded timestamp with time zone NOT NULL DEFAULT current_timestamp,
            created timestamp with time zone,
            location geography(point)
        )
    ",
            &[],
        )
        .await
        .unwrap();
    Ok(())
}
