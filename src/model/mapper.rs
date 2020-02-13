use anyhow::anyhow;
use chrono::{DateTime, Utc};
use deunicode::deunicode;
use futures::{Stream, StreamExt};
use regex::Regex;
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};
use tokio_postgres::types::ToSql;
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
    pub size: u64,
    pub sha3: [u8; 32],
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
    pub const COLS: [&'static str; 10] = [
        "id",
        "media_type",
        "path",
        "thumbnail_path",
        "preview_path",
        "size",
        "sha3",
        "uploaded",
        "created",
        "location",
    ];

    pub async fn insert<P1, P2, P3>(
        client: &Client,
        media_type: EntityType,
        path: P1,
        thumbnail_path: P2,
        preview_path: P3,
        size: u64,
        sha3: &[u8; 32],
        created: &Option<DateTime<Utc>>,
        location: &Option<Location>,
    ) -> Result<Self>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        P3: AsRef<Path>,
    {
        Ok(Self::from_row(
            &client
                .query_one(
                    format!(
                        "
                            INSERT INTO entity(
                                media_type,
                                path,
                                thumbnail_path,
                                preview_path,
                                size,
                                sha3,
                                created,
                                location
                            )
                            VALUES(
                                $1,
                                $2,
                                $3,
                                $4,
                                $5,
                                $6,
                                $7,
                                $8
                            )
                            RETURNING {}
                        ",
                        Self::COLS.join(", "),
                    )
                    .as_str(),
                    &[
                        &media_type,
                        &path
                            .as_ref()
                            .to_str()
                            .ok_or(anyhow!("Media path contains non UTF-8 characters"))?,
                        &thumbnail_path
                            .as_ref()
                            .to_str()
                            .ok_or(anyhow!("Thumbnial path contains non UTF-8 characters"))?,
                        &preview_path
                            .as_ref()
                            .to_str()
                            .ok_or(anyhow!("Preview path contains non UTF-8 characters"))?,
                        &i64::try_from(size)?,
                        &sha3.as_ref(),
                        &created,
                        &location,
                    ],
                )
                .await?,
        )?)
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get::<_, i32>(0)?,
            media_type: row.try_get::<_, EntityType>(1)?,
            path: Path::new(row.try_get::<_, &str>(2)?).to_path_buf(),
            thumbnail_path: Path::new(row.try_get::<_, &str>(3)?).to_path_buf(),
            preview_path: Path::new(row.try_get::<_, &str>(4)?).to_path_buf(),
            size: row.try_get::<_, i64>(5)?.try_into()?,
            sha3: row.try_get::<_, &[u8]>(6)?.try_into()?,
            uploaded: row.try_get::<_, DateTime<Utc>>(7)?,
            created: row.try_get::<_, Option<DateTime<Utc>>>(8)?,
            location: row.try_get::<_, Option<Location>>(9)?,
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

    pub async fn get_from_sha3(client: &Client, sha3: &[u8; 32]) -> Option<Self> {
        let row = client
            .query_opt(
                format!(
                    "SELECT {} FROM entity WHERE sha3 = $1",
                    Self::COLS.join(", ")
                )
                .as_str(),
                &[&sha3.as_ref()],
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

    pub fn canonical_name(name: &str) -> Result<String> {
        let re_space = Regex::new(r"\s+")?;
        let name = deunicode(name);
        let name = re_space.replace_all(&name, "-");
        Ok(name.to_lowercase())
    }

    pub async fn insert(
        client: &Client,
        name: &str,
        tag_type: &str,
        parent: Option<String>,
    ) -> Result<Self> {
        let tag = TagType::try_from(tag_type)?;
        let pid = match parent {
            None => None,
            Some(parent) => Some(
                Tag::get_from_canonical_name(client, parent.as_str())
                    .await
                    .ok_or(anyhow!("Parent {} does not exist", parent))?
                    .id,
            ),
        };
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
                    &[&pid, &Self::canonical_name(&name)?, &name, &tag],
                )
                .await?,
        )?)
    }

    pub async fn list(client: &Client) -> Result<impl Stream<Item = Result<Self>>> {
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

    pub async fn get<T: Borrow<i32>>(client: &Client, id: T) -> Option<Self> {
        let row = client
            .query_opt(
                format!("SELECT {} FROM tag WHERE id = $1", Self::COLS.join(", ")).as_str(),
                &[id.borrow()],
            )
            .await
            .ok()
            .flatten()?;
        Self::from_row(&row).ok()
    }

    pub async fn search(
        client: &Client,
        tags: &Vec<String>,
    ) -> Result<impl Stream<Item = Result<Entity>>> {
        let list: Vec<_> = tags
            .iter()
            .enumerate()
            .map(|(i, _)| {
                format!(
                    "
                        SELECT e.{}
                        FROM (
                            WITH RECURSIVE deeptag AS (
                                SELECT * FROM tag WHERE canonical_name = ${}
                                UNION
                                SELECT t.* FROM tag t JOIN deeptag dt ON dt.id = t.pid
                            )
                            SELECT * FROM deeptag
                        ) t, tag_to_entity t2e, entity e
                        WHERE t.id = t2e.tid AND t2e.eid = e.id
                    ",
                    Entity::COLS.join(", e."),
                    i + 1
                )
            })
            .collect();
        let query = list.join(" INTERSECT ");
        Ok(client
            .query_raw(query.as_str(), tags.iter().map(|x| x as &dyn ToSql))
            .await?
            .map(|row| Ok(Entity::from_row(&row?)?)))
    }

    pub async fn save(&self, client: &Client) -> Result<()> {
        client
            .query_one(
                format!(
                    "
                        UPDATE tag
                        SET pid = $1,
                            name = $2,
                            canonical_name = $3,
                            type = $4
                        WHERE id = $5
                        RETURNING {}
                    ",
                    Self::COLS.join(", "),
                )
                .as_str(),
                &[
                    &self.pid,
                    &self.name,
                    &self.canonical_name,
                    &self.tag_type,
                    &self.id,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn get_from_eid<'a, T: Borrow<i32> + 'a>(
        client: &Client,
        eid: T,
    ) -> Result<impl Stream<Item = Result<Self>>> {
        Ok(client
            .query_raw(
                format!(
                    "
                    SELECT {} FROM tag t JOIN tag_to_entity t2e on t.id = t2e.tid WHERE t2e.eid = $1",
                    Self::COLS.join(", ")
                )
                .as_str(),
                vec![eid.borrow()].iter().map(|x| x as &dyn ToSql),
            )
            .await?
            .map(|row| Ok(Self::from_row(&row?)?)))
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
