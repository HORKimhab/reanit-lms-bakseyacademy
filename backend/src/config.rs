use anyhow::Context;

use crate::{
    storage::ObjectStorage,
    telegram::{LocalDir, Telegram},
    video::BunnyStream,
};

pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    /// Lesson video provider; `None` (no `BUNNY_LIBRARY_ID`) means lessons have no playable URL.
    pub video: Option<BunnyStream>,
    /// Ebook file bucket; `None` (no `STORAGE_BUCKET`) means downloads are unavailable.
    pub storage: Option<ObjectStorage>,
    /// HMAC key for `/media/{token}` links (`MEDIA_SIGNING_KEY`; required with Telegram, random otherwise
    /// since nothing is signed then).
    pub media_key: Vec<u8>,
    /// Telegram channel storage; `None` without `TELEGRAM_BOT_TOKEN`.
    pub telegram: Option<Telegram>,
    /// Unpaid (`pending_payment`) orders older than this many days are cancelled (`ORDER_ABANDON_DAYS`,
    /// default 7).
    pub order_abandon_days: i32,
}

/// Unset or blank → `None`.
fn optional(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let media_key = optional("MEDIA_SIGNING_KEY");
        if media_key.is_none() && optional("TELEGRAM_BOT_TOKEN").is_some() {
            anyhow::bail!("MEDIA_SIGNING_KEY must be set with TELEGRAM_BOT_TOKEN (it signs /media links)");
        }
        let order_abandon_days = match optional("ORDER_ABANDON_DAYS") {
            None => 7,
            Some(v) => v
                .parse::<i32>()
                .ok()
                .filter(|d| (1..=365).contains(d))
                .context("ORDER_ABANDON_DAYS must be a number of days between 1 and 365")?,
        };
        Ok(Self {
            order_abandon_days,
            media_key: media_key.map(String::into_bytes).unwrap_or_else(|| {
                use argon2::password_hash::rand_core::{OsRng, RngCore};
                let mut key = vec![0u8; 32];
                OsRng.fill_bytes(&mut key);
                key
            }),
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            video: optional("BUNNY_LIBRARY_ID").map(|library_id| BunnyStream {
                library_id,
                token_key: optional("BUNNY_TOKEN_KEY"),
            }),
            telegram: match optional("TELEGRAM_BOT_TOKEN") {
                None => None,
                Some(token) => Some(Telegram::new(
                    &optional("TELEGRAM_API_URL").unwrap_or_else(|| "https://api.telegram.org".into()),
                    token,
                    optional("TELEGRAM_CHANNEL_ID").context("TELEGRAM_CHANNEL_ID must be set with TELEGRAM_BOT_TOKEN")?,
                    optional("TELEGRAM_LOCAL_DIR").map(|host_dir| LocalDir {
                        server_dir: optional("TELEGRAM_SERVER_DIR").unwrap_or_else(|| "/var/lib/telegram-bot-api".into()),
                        host_dir: host_dir.into(),
                    }),
                )
                .with_webhook_secret(optional("TELEGRAM_WEBHOOK_SECRET"))
                .with_admin_chat(optional("TELEGRAM_ADMIN_CHAT_ID"))),
            },
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
