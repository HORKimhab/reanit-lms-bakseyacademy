use anyhow::Context;

use crate::{storage::ObjectStorage, video::BunnyStream};

pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    /// Lesson video provider; `None` (no `BUNNY_LIBRARY_ID`) means lessons have no playable URL.
    pub video: Option<BunnyStream>,
    /// Ebook file bucket; `None` (no `STORAGE_BUCKET`) means downloads are unavailable.
    pub storage: Option<ObjectStorage>,
}

/// Unset or blank → `None`.
fn optional(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            video: optional("BUNNY_LIBRARY_ID").map(|library_id| BunnyStream {
                library_id,
                token_key: optional("BUNNY_TOKEN_KEY"),
            }),
            storage: match optional("STORAGE_BUCKET") {
                None => None,
                Some(bucket) => Some(ObjectStorage {
                    endpoint: optional("STORAGE_ENDPOINT").context("STORAGE_ENDPOINT must be set with STORAGE_BUCKET")?,
                    bucket,
                    region: optional("STORAGE_REGION").unwrap_or_else(|| "auto".into()),
                    access_key_id: optional("STORAGE_ACCESS_KEY_ID").context("STORAGE_ACCESS_KEY_ID must be set")?,
                    secret_access_key: optional("STORAGE_SECRET_ACCESS_KEY")
                        .context("STORAGE_SECRET_ACCESS_KEY must be set")?,
                    path_style: optional("STORAGE_PATH_STYLE").is_none_or(|v| v != "false"),
                }),
            },
        })
    }
}
