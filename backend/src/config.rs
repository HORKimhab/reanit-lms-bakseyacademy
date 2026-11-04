use anyhow::Context;

use std::sync::Arc;

use crate::{
    sms::{DevLog, SmsSender, Twilio, TwilioSender},
    payments::{self, Payments, bakong, khqr, payway},
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
    /// Automatic payment confirmation; each provider is off unless its credentials are set.
    pub payments: Payments,
    /// Password-reset texts: Twilio, or `SMS_DEV_LOG=true` to log them (development only).
    pub sms: Option<Arc<dyn SmsSender>>,
}

fn sms_from_env() -> anyhow::Result<Option<Arc<dyn SmsSender>>> {
    if let Some(account_sid) = optional("TWILIO_ACCOUNT_SID") {
        let auth_token = optional("TWILIO_AUTH_TOKEN").context("TWILIO_AUTH_TOKEN must be set with TWILIO_ACCOUNT_SID")?;
        let sender = match (optional("TWILIO_MESSAGING_SERVICE_SID"), optional("TWILIO_FROM")) {
            (Some(sid), _) => TwilioSender::MessagingService(sid),
            (None, Some(from)) => TwilioSender::From(from),
            (None, None) => anyhow::bail!("set TWILIO_FROM or TWILIO_MESSAGING_SERVICE_SID with TWILIO_ACCOUNT_SID"),
        };
        return Ok(Some(Arc::new(Twilio::new(account_sid, auth_token, sender))));
    }
    if optional("SMS_DEV_LOG").is_some_and(|v| v == "true") {
        tracing::warn!("SMS_DEV_LOG=true: password reset codes are written to the log, never use this in production");
        return Ok(Some(Arc::new(DevLog)));
    }
    Ok(None)
}

/// `SITE_URL`: the public storefront origin providers send buyers and pushbacks back to.
/// Must be `https://` (plain `http://` only for localhost while developing).
fn site_url() -> anyhow::Result<String> {
    let url = optional("SITE_URL").context("SITE_URL must be set with PAYWAY_MERCHANT_ID (e.g. https://reanit.com)")?;
    let url = url.trim_end_matches('/').to_owned();
    let local = url.starts_with("http://localhost") || url.starts_with("http://127.0.0.1");
    if !(url.starts_with("https://") || local) || url.contains(char::is_whitespace) {
        anyhow::bail!("SITE_URL must be an https:// origin");
    }
    Ok(url)
}

/// Bakong account and labels, checked against the KHQR field limits.
fn bakong_merchant() -> anyhow::Result<khqr::Merchant> {
    let account_id = optional("BAKONG_ACCOUNT_ID").context("BAKONG_ACCOUNT_ID must be set with BAKONG_TOKEN")?;
    let name = optional("BAKONG_MERCHANT_NAME").context("BAKONG_MERCHANT_NAME must be set with BAKONG_TOKEN")?;
    let city = optional("BAKONG_MERCHANT_CITY").unwrap_or_else(|| "Phnom Penh".into());
    let ascii = |s: &str| s.is_ascii() && !s.chars().any(|c| c.is_control());
    if account_id.len() > 32 || !account_id.contains('@') || !ascii(&account_id) {
        anyhow::bail!("BAKONG_ACCOUNT_ID must look like name@bank (at most 32 characters)");
    }
    if name.is_empty() || name.len() > 25 || !ascii(&name) || city.len() > 15 || !ascii(&city) {
        anyhow::bail!("BAKONG_MERCHANT_NAME (≤ 25) and BAKONG_MERCHANT_CITY (≤ 15) must be plain ASCII");
    }
    let merchant_id = optional("BAKONG_MERCHANT_ID");
    let acquiring_bank = optional("BAKONG_ACQUIRING_BANK");
    if merchant_id.is_some() != acquiring_bank.is_some() {
        anyhow::bail!("BAKONG_MERCHANT_ID and BAKONG_ACQUIRING_BANK go together");
    }
    Ok(khqr::Merchant { account_id, name, city, merchant_id, acquiring_bank })
}

fn payments_from_env() -> anyhow::Result<Payments> {
    let bakong = match optional("BAKONG_TOKEN") {
        None => None,
        Some(token) => Some(payments::Bakong {
            merchant: bakong_merchant()?,
            gateway: Arc::new(bakong::Client::new(
                optional("BAKONG_API_URL").unwrap_or_else(|| "https://api-bakong.nbc.gov.kh".into()),
                token,
            )),
        }),
    };
    let payway = match optional("PAYWAY_MERCHANT_ID") {
        None => None,
        Some(merchant_id) => {
            let config = payway::Config {
                merchant_id,
                api_key: optional("PAYWAY_API_KEY").context("PAYWAY_API_KEY must be set with PAYWAY_MERCHANT_ID")?,
                api_url: optional("PAYWAY_API_URL")
                    .context("PAYWAY_API_URL must be set with PAYWAY_MERCHANT_ID (sandbox: https://checkout-sandbox.payway.com.kh)")?
                    .trim_end_matches('/')
                    .to_owned(),
            };
            if !config.api_url.starts_with("https://") {
                anyhow::bail!("PAYWAY_API_URL must be https://");
            }
            Some(payments::Payway { gateway: Arc::new(payway::Client::new(config.clone())), config, site_url: site_url()? })
        }
    };
    Ok(Payments { bakong, payway })
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
            payments: payments_from_env()?,
            sms: sms_from_env()?,
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
