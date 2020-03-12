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
use tokio_postgres::Client;

use crate::error::Result;

pub async fn create_schema(client: &Client) -> Result<()> {
    // TODO: Transaction hanlding?
    client
        .execute("CREATE EXTENSION IF NOT EXISTS postgis", &[])
        .await?;
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
        .await?;
    client
        .execute(
            "
                CREATE TABLE IF NOT EXISTS entity(
                    id serial PRIMARY KEY NOT NULL,
                    media_type entity_type NOT NULL,
                    path varchar NOT NULL,
                    thumbnail_path varchar NOT NULL,
                    preview_path varchar NOT NULL,
                    size int8 NOT NULL,
                    sha3 bytea NOT NULL,
                    uploaded timestamp with time zone NOT NULL DEFAULT current_timestamp,
                    created timestamp with time zone,
                    location geography(point),
                    unique (sha3)
                )
            ",
            &[],
        )
        .await?;
    client
        .execute(
            "
                DO $$ BEGIN
                    CREATE TYPE tag_type AS ENUM ('person', 'place', 'event', 'other');
                EXCEPTION
                    WHEN duplicate_object THEN null;
                END $$
            ",
            &[],
        )
        .await?;
    client
        .execute(
            "
                CREATE TABLE IF NOT EXISTS tag(
                    id serial PRIMARY KEY NOT NULL,
                    pid integer references tag(id),
                    canonical_name varchar NOT NULL,
                    name varchar NOT NULL,
                    unique (canonical_name)
                )
            ",
            &[],
        )
        .await?;
    client
        .execute(
            "
                CREATE TABLE IF NOT EXISTS tag_to_entity(
                    tid integer NOT NULL references tag(id),
                    eid integer NOT NULL references entity(id),
                    unique (tid, eid)
                )
            ",
            &[],
        )
        .await?;
    Ok(())
}
