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
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use deunicode::deunicode;
use futures::{Stream, StreamExt, TryStreamExt};
use regex::Regex;
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, Row};

use super::types::EntityType;
use crate::coord::Location;
use crate::error::Result;
use crate::hash::Sha3;

#[derive(Debug, PartialEq)]
pub struct Entity {
    pub id: i32,
    pub media_type: EntityType,
    pub path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub preview_path: PathBuf,
    pub uploaded: DateTime<Utc>,
    pub size: u64,
    pub sha3: Sha3,
    pub created: Option<DateTime<Utc>>,
    pub location: Option<Location>,
}

#[derive(Debug, PartialEq)]
pub struct Tag {
    pub id: i32,
    pub pid: Option<i32>,
    pub canonical_name: String,
    pub name: String,
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
        sha3: &Sha3,
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

    pub async fn save(&self, client: &Client) -> Result<()> {
        client
            .query_one(
                format!(
                    "
                        UPDATE entity
                        SET media_type = $1,
                            path = $2,
                            thumbnail_path = $3,
                            preview_path = $4,
                            size = $5,
                            sha3 = $6,
                            uploaded = $7,
                            created = $8,
                            location = $9
                        WHERE id = $10
                        RETURNING {}
                    ",
                    Self::COLS.join(", "),
                )
                .as_str(),
                &[
                    &self.media_type,
                    &self
                        .path
                        .to_str()
                        .ok_or(anyhow!("Media path contains non UTF-8 characters"))?,
                    &self
                        .thumbnail_path
                        .to_str()
                        .ok_or(anyhow!("Thumbnial path contains non UTF-8 characters"))?,
                    &self
                        .preview_path
                        .to_str()
                        .ok_or(anyhow!("Preview path contains non UTF-8 characters"))?,
                    &i64::try_from(self.size)?,
                    &self.sha3.as_ref(),
                    &self.uploaded,
                    &self.created,
                    &self.location,
                    &self.id,
                ],
            )
            .await?;
        Ok(())
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get::<_, i32>(0)?,
            media_type: row.try_get::<_, EntityType>(1)?,
            path: Path::new(row.try_get::<_, &str>(2)?).to_path_buf(),
            thumbnail_path: Path::new(row.try_get::<_, &str>(3)?).to_path_buf(),
            preview_path: Path::new(row.try_get::<_, &str>(4)?).to_path_buf(),
            size: row.try_get::<_, i64>(5)?.try_into()?,
            sha3: Sha3::try_from_slice(row.try_get::<_, &[u8]>(6)?)?,
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

    pub async fn get_from_sha3(client: &Client, sha3: &Sha3) -> Option<Self> {
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

    pub async fn delete<T: Borrow<i32>>(client: &Client, id: T) -> Result<()> {
        let num_rows = &client
            .execute(
                "
            DELETE FROM entity
            WHERE id = $1
            ",
                &[id.borrow()],
            )
            .await?;
        if num_rows != &1 {
            return Err(anyhow!("No such image for the given id").into());
        }
        Ok(())
    }
}

impl Tag {
    pub const COLS: [&'static str; 4] = ["id", "pid", "canonical_name", "name"];

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get::<_, i32>(0)?,
            pid: row.try_get::<_, Option<i32>>(1)?,
            canonical_name: row.try_get::<_, String>(2)?,
            name: row.try_get::<_, String>(3)?,
        })
    }

    pub fn canonical_name(name: &str) -> Result<String> {
        let re_space = Regex::new(r"\s+")?;
        let name = deunicode(name);
        let name = re_space.replace_all(&name, "-");
        Ok(name.to_lowercase())
    }

    pub async fn insert(client: &Client, name: &str, parent: Option<String>) -> Result<Self> {
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
                            INSERT INTO tag(pid, canonical_name, name)
                            VALUES($1, $2, $3)
                            RETURNING {}
                        ",
                        Self::COLS.join(", "),
                    )
                    .as_str(),
                    &[&pid, &Self::canonical_name(&name)?, &name],
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

    pub async fn list_from_ids(client: &Client, ids: &[i32]) -> Result<Vec<Self>> {
        Ok(client
            .query_raw(
                format!(
                    "SELECT {} FROM tag WHERE id IN (select(unnest($1::int[])))",
                    Self::COLS.join(", ")
                )
                .as_str(),
                vec![ids].iter().map(|x| x as &dyn ToSql),
            )
            .await?
            .map(|row| -> Result<Self> { Ok(Self::from_row(&row?)?) })
            .try_collect()
            .await?)
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
        let query = format!(
            "
                SELECT {}
                FROM (
                    SELECT DISTINCT ON(id) {}
                    FROM ({}) e
                    ORDER BY id
                ) e
                ORDER BY created
            ",
            Entity::COLS.join(", "),
            Entity::COLS.join(", "),
            list.join(" INTERSECT "),
        );
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
                &[&self.pid, &self.name, &self.canonical_name, &self.id],
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

    pub async fn insert<T: Borrow<i32>, E: Borrow<i32>>(
        client: &Client,
        tid: T,
        eid: E,
    ) -> Result<Self> {
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
                    &[tid.borrow(), eid.borrow()],
                )
                .await?,
        )?)
    }

    pub async fn delete<T: Borrow<i32>, E: Borrow<i32>>(
        client: &Client,
        tid: T,
        eid: E,
    ) -> Result<()> {
        let num_rows = &client
            .execute(
                "
                    DELETE FROM tag_to_entity
                    WHERE tid = $1 AND eid = $2
                ",
                &[tid.borrow(), eid.borrow()],
            )
            .await?;
        if num_rows != &1 {
            return Err(anyhow!("No such tag for the given entity").into());
        }
        Ok(())
    }
}
