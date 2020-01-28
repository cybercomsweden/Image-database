use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};
use regex::Regex;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use tokio_postgres::{Client, Row};

use super::types::{EntityType, TagType};
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

#[derive(Debug, PartialEq)]
pub struct Tag {
    pub id: i32,
    pub pid: Option<i32>,
    pub canonical_name: String,
    pub name: String,
    pub tag_type: TagType,
}

#[derive(Debug, PartialEq)]
pub struct TagToEntity {
    pub tid: i32,
    pub eid: i32,
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

    pub async fn get_from_path<T: Borrow<str>>(client: &Client, path: T) -> Option<Self> {
        let row = client
            .query_opt(
                format!(
                    "SELECT {} FROM entity WHERE path = $1",
                    Self::COLS.join(", ")
                )
                .as_str(),
                &[&path.borrow()],
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

impl Tag {
    pub const COLS: [&'static str; 5] = ["id", "pid", "canonical_name", "name", "type"];

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get::<_, i32>(0)?,
            pid: row.try_get::<_, Option<i32>>(1)?,
            canonical_name: row.try_get::<_, String>(2)?,
            name: row.try_get::<_, String>(3)?,
            tag_type: row.try_get::<_, TagType>(4)?,
        })
    }

    fn canonical_name(name: &str) -> Result<String> {
        // NOTE: only tag in english atm
        let re_char = Regex::new(r"[^A-Za-z0-9\s]")?;
        let re_space = Regex::new(r"\s+")?;
        let name = re_char.replace_all(&name, "");
        let name = re_space.replace_all(&name, "-");
        Ok(name.to_lowercase())
    }

    pub async fn insert(
        client: &Client,
        name: &str,
        tag_type: &str,
        parent: Option<i32>,
    ) -> Result<Self> {
        let tag = TagType::try_from(tag_type)?;
        Ok(Self::from_row(
            &client
                .query_one(
                    format!(
                        "
                        INSERT INTO tag(pid, canonical_name, name, type)
                        VALUES($1, $2, $3, $4)
                        RETURNING {}
                    ",
                        Self::COLS.join(", "),
                    )
                    .as_str(),
                    &[&parent, &Self::canonical_name(&name)?, &name, &tag],
                )
                .await?,
        )?)
    }

    pub async fn list(client: &Client) -> Result<impl Stream<Item = Result<Self>>> {
        //Ok(client
        //    .query_raw(
        //        format!(
        //            "
        //            WITH RECURSIVE recursetree AS (
        //                SELECT * FROM tag WHERE pid = NULL
        //              UNION
        //                SELECT t.*
        //                FROM tag t
        //                JOIN recursetree rt ON rt.id = t.pid
        //            )
        //            SELECT *
        //            FROM recursetree
        //            RETURNING {}
        //       ",
        //            Self::COLS.join(", "),
        //        )
        //        .as_str(),
        //        vec![],
        //    )
        //    .await?
        //    .map(|row| Ok(Self::from_row(&row?)?)))
        Ok(client
            .query_raw(
                format!("SELECT {} FROM tag", Self::COLS.join(", ")).as_str(),
                vec![],
            )
            .await?
            .map(|row| Ok(Self::from_row(&row?)?)))
    }

    pub async fn get_from_canonical_name<T: Borrow<str>>(
        client: &Client,
        canonical_name: T,
    ) -> Option<Self> {
        let row = client
            .query_opt(
                format!(
                    "SELECT {} FROM tag WHERE canonical_name = $1",
                    Self::COLS.join(", ")
                )
                .as_str(),
                &[&canonical_name.borrow()],
            )
            .await
            .ok()
            .flatten()?;
        Self::from_row(&row).ok()
    }
}

impl TagToEntity {
    pub const COLS: [&'static str; 2] = ["tid", "eid"];

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            tid: row.try_get::<_, i32>(0)?,
            eid: row.try_get::<_, i32>(1)?,
        })
    }

    pub async fn insert(client: &Client, tid: &i32, eid: &i32) -> Result<Self> {
        Ok(Self::from_row(
            &client
                .query_one(
                    format!(
                        "
                        INSERT INTO tag_to_entity(tid, eid)
                        VALUES($1, $2)
                        RETURNING {}
                    ",
                        Self::COLS.join(", "),
                    )
                    .as_str(),
                    &[&tid, &eid],
                )
                .await?,
        )?)
    }
}
