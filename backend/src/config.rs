use anyhow::Context;

use crate::video::BunnyStream;

pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    /// Lesson video provider; `None` (no `BUNNY_LIBRARY_ID`) means lessons have no playable URL.
    pub video: Option<BunnyStream>,
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
        })
    }
}
