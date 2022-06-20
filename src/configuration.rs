use config::{Config, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    application: ApplicationSettings,
    database: DatabaseSettings,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApplicationSettings {
    cookie: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    userid: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DatabaseSettings {
    username: String,
    password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    port: u16,
    host: String,
    name: String,
}

impl ApplicationSettings {
    pub(crate) fn cookie(&self) -> &str {
        &self.cookie
    }

    pub(crate) fn userid(&self) -> i32 {
        self.userid
    }
}

impl DatabaseSettings {
    pub(crate) fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(PgSslMode::Prefer)
    }

    pub(crate) fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.name)
    }
}

impl Settings {
    pub(crate) fn new() -> Result<Self, anyhow::Error> {
        Config::builder()
            .add_source(File::with_name("./config.yaml").required(true))
            .build()
            .and_then(|config| config.try_deserialize())
            .map_err(Into::into)
    }

    pub(crate) fn application(&self) -> &ApplicationSettings {
        &self.application
    }

    pub(crate) fn database(&self) -> &DatabaseSettings {
        &self.database
    }
}
