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
                    uploaded timestamp with time zone NOT NULL DEFAULT current_timestamp,
                    created timestamp with time zone,
                    location geography(point)
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
                    type tag_type NOT NULL,
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
