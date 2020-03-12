/*
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
*/
use anyhow::anyhow;
use futures::FutureExt;
use std::collections::BTreeSet;
use std::path::Path;
use tokio_postgres::{Client, Config as PostgresConfig, NoTls};

use crate::config::Config;
use crate::error::Result;
use crate::model::EntityType;
use crate::thumbnail::{file_type_from_path, MediaType};

pub type DbConn = Client;

pub fn get_media_type<P: AsRef<Path>>(path: P) -> Result<EntityType> {
    let file_type = file_type_from_path(path).ok_or(anyhow!("Unknown file type"))?;
    match file_type.media_type() {
        MediaType::Image | MediaType::RawImage => Ok(EntityType::Image),
        MediaType::Video => Ok(EntityType::Video),
    }
}

/// Helper method to access database the database in a request handler. Use by
/// adding `db: web::Data<DbConn>` to your request handler's argument list.
pub async fn get_db(config: Config) -> Result<DbConn> {
    // TODO: This panics if it's unable to connect to database. How to handle?

    // Create a client that we use to query the database and a connection that
    // we use to wake up the futures when we query the database
    let (client, conn) = PostgresConfig::new()
        .host(&config.database.host)
        .port(config.database.port)
        .user(&config.database.user)
        .dbname(&config.database.dbname)
        .connect(NoTls)
        .await?;

    // We must provide the event loop with our connection, or our query futures
    // will never resolve
    actix_rt::spawn(conn.map(|_| ()));

    Ok(client)
}

pub fn get_differences<'a, T, F: Fn(&T) -> U, U: Ord>(
    curr: &'a [T],
    new: &'a [T],
    key: F,
) -> (Vec<&'a T>, Vec<&'a T>) {
    let curr_set: BTreeSet<U> = curr.iter().map(&key).collect();
    let new_set: BTreeSet<U> = new.iter().map(&key).collect();

    let mut to_add = Vec::new();
    for to_add_key in new_set.difference(&curr_set) {
        for v in new {
            if to_add_key == &key(v) {
                to_add.push(v);
            }
        }
    }

    let mut to_remove = Vec::new();
    for to_remove_key in curr_set.difference(&new_set) {
        for v in curr {
            if to_remove_key == &key(v) {
                to_remove.push(v);
            }
        }
    }

    (to_add, to_remove)
}
