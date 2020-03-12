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
