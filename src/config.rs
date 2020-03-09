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
use serde::Deserialize;

fn db_default_host() -> String {
    "/var/run/postgresql/".into()
}

fn db_default_port() -> u16 {
    5432
}

fn db_default_user() -> String {
    // TODO: Error handling
    let output = std::process::Command::new("whoami").output().unwrap();
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .trim_end()
        .into()
}

fn db_default_dbname() -> String {
    "backlog".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    #[serde(default = "db_default_host")]
    pub host: String,
    #[serde(default = "db_default_port")]
    pub port: u16,
    #[serde(default = "db_default_user")]
    pub user: String,
    #[serde(default = "db_default_dbname")]
    pub dbname: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub database: Database,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            host: db_default_host(),
            port: db_default_port(),
            user: db_default_user(),
            dbname: db_default_dbname(),
        }
    }
}
