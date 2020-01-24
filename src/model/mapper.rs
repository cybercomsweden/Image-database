use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};
use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use tokio_postgres::{Client, Row};

use super::types::EntityType;
use crate::coord::Location;
use crate::error::Result;

#[derive(Debug, PartialEq)]
pub struct Entity {
    pub id: i32,
    pub media_type: EntityType,
    pub path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub preview_path: PathBuf,
    pub uploaded: DateTime<Utc>,
    pub created: Option<DateTime<Utc>>,
    pub location: Option<Location>,
}

impl Entity {
    pub const COLS: [&'static str; 8] = [
        "id",
        "media_type",
        "path",
        "thumbnail_path",
        "preview_path",
        "uploaded",
        "created",
        "location",
    ];

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get::<_, i32>(0)?,
            media_type: row.try_get::<_, EntityType>(1)?,
            path: Path::new(row.try_get::<_, &str>(2)?).to_path_buf(),
            thumbnail_path: Path::new(row.try_get::<_, &str>(3)?).to_path_buf(),
            preview_path: Path::new(row.try_get::<_, &str>(4)?).to_path_buf(),
            uploaded: row.try_get::<_, DateTime<Utc>>(5)?,
            created: row.try_get::<_, Option<DateTime<Utc>>>(6)?,
            location: row.try_get::<_, Option<Location>>(7)?,
        })
    }

    pub async fn get<T: Borrow<i32>>(client: &Client, id: T) -> Option<Self> {
        let row = client
            .query_opt(
                format!("SELECT {} FROM entity WHERE id = $1", Self::COLS.join(", ")).as_str(),
                &[id.borrow()],
            )
            .await
            .ok()
            .flatten()?;
        Self::from_row(&row).ok()
    }

    pub async fn list_desc(client: &Client) -> Result<impl Stream<Item = Result<Self>>> {
        Ok(client
            .query_raw(
                format!(
                    "SELECT {} FROM entity ORDER BY id DESC",
                    Self::COLS.join(", ")
                )
                .as_str(),
                vec![],
            )
            .await?
            .map(|row| Ok(Self::from_row(&row?)?)))
    }
}
