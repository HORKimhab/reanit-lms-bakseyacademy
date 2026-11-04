//! Outgoing SMS (password-reset codes) behind [`SmsSender`], so tests can capture texts.

use std::time::Duration;

use anyhow::Context;
use futures_util::future::BoxFuture;

pub trait SmsSender: Send + Sync {
    /// Sends `text` to `to` (E.164).
    fn send<'a>(&'a self, to: &'a str, text: &'a str) -> BoxFuture<'a, anyhow::Result<()>>;
}

/// Twilio Programmable Messaging (`TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, and `TWILIO_FROM` or
/// `TWILIO_MESSAGING_SERVICE_SID`).
pub struct Twilio {
    pub account_sid: String,
    pub auth_token: String,
    /// `From` number / alphanumeric sender id, or a `MG…` messaging service sid.
    pub sender: TwilioSender,
    pub http: reqwest::Client,
}

pub enum TwilioSender {
    From(String),
    MessagingService(String),
}

impl Twilio {
    pub fn new(account_sid: String, auth_token: String, sender: TwilioSender) -> Self {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(15)).build().expect("http client");
        Self { account_sid, auth_token, sender, http }
    }
}

impl SmsSender for Twilio {
    fn send<'a>(&'a self, to: &'a str, text: &'a str) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let sender = match &self.sender {
                TwilioSender::From(from) => ("From", from.as_str()),
                TwilioSender::MessagingService(sid) => ("MessagingServiceSid", sid.as_str()),
            };
            let res = self
                .http
                .post(format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", self.account_sid))
                .basic_auth(&self.account_sid, Some(&self.auth_token))
                .form(&[("To", to), sender, ("Body", text)])
                .send()
                .await
                .context("twilio: request")?;
            let status = res.status();
            if !status.is_success() {
                // Twilio's error body names the problem (never echoes the auth token).
                let body = res.text().await.unwrap_or_default();
                anyhow::bail!("twilio: HTTP {status}: {}", body.chars().take(300).collect::<String>());
            }
            Ok(())
        })
    }
}

/// Development only (`SMS_DEV_LOG=true`): writes texts to the API log instead of sending them.
pub struct DevLog;

impl SmsSender for DevLog {
    fn send<'a>(&'a self, to: &'a str, text: &'a str) -> BoxFuture<'a, anyhow::Result<()>> {
        tracing::warn!(to, text, "SMS_DEV_LOG: not sent");
        Box::pin(async { Ok(()) })
    }
}
