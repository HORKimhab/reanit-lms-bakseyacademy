//! Google sign-in (OpenID Connect authorization-code flow with PKCE). The storefront keeps
//! `state` / `nonce` / the PKCE verifier in an HttpOnly cookie; the API holds the client secret,
//! exchanges the code and checks the ID token's claims.

use std::{sync::Arc, time::Duration};

use anyhow::Context;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use futures_util::future::BoxFuture;
use serde::Deserialize;
use serde_json::Value;

/// Exchanges an authorization code for an ID token at Google's token endpoint.
pub trait GoogleTokens: Send + Sync {
    fn exchange<'a>(&'a self, code: &'a str, verifier: &'a str, redirect_uri: &'a str) -> BoxFuture<'a, anyhow::Result<String>>;
}

#[derive(Clone)]
pub struct Google {
    pub client_id: String,
    /// `{SITE_URL}/auth/google/callback`, registered in the Google Cloud console.
    pub redirect_uri: String,
    pub tokens: Arc<dyn GoogleTokens>,
}

pub struct HttpTokens {
    pub client_id: String,
    pub client_secret: String,
    pub http: reqwest::Client,
}

impl HttpTokens {
    pub fn new(client_id: String, client_secret: String) -> Self {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(15)).build().expect("http client");
        Self { client_id, client_secret, http }
    }
}

impl GoogleTokens for HttpTokens {
    fn exchange<'a>(&'a self, code: &'a str, verifier: &'a str, redirect_uri: &'a str) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async move {
            let res = self
                .http
                .post("https://oauth2.googleapis.com/token")
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("code", code),
                    ("code_verifier", verifier),
                    ("redirect_uri", redirect_uri),
                    ("client_id", &self.client_id),
                    ("client_secret", &self.client_secret),
                ])
                .send()
                .await
                .context("google: token request")?;
            let status = res.status();
            let body: Value = res.json().await.context("google: token response not JSON")?;
            if !status.is_success() {
                anyhow::bail!("google: HTTP {status}: {}", body["error"].as_str().unwrap_or("error"));
            }
            body["id_token"].as_str().map(str::to_owned).context("google: no id_token")
        })
    }
}

/// RFC 3986 percent-encoding of everything but unreserved characters.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

impl Google {
    pub fn authorize_url(&self, state: &str, nonce: &str, code_challenge: &str) -> String {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", "openid email profile"),
            ("state", state),
            ("nonce", nonce),
            ("code_challenge", code_challenge),
            ("code_challenge_method", "S256"),
            ("prompt", "select_account"),
        ];
        let query: Vec<String> = params.iter().map(|(k, v)| format!("{k}={}", encode(v))).collect();
        format!("https://accounts.google.com/o/oauth2/v2/auth?{}", query.join("&"))
    }
}

#[derive(Deserialize)]
pub struct Claims {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub exp: i64,
    pub iat: Option<i64>,
    pub nonce: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub email_verified: bool,
    pub name: Option<String>,
}

/// Decodes the ID token received directly from Google's token endpoint over TLS and checks its
/// claims (OIDC Core 3.1.3.7: with direct TLS the signature check may be skipped; issuer,
/// audience, expiry and nonce still have to match). `None` when anything is off.
pub fn verify_id_token(id_token: &str, client_id: &str, nonce: &str, now: i64) -> Option<Claims> {
    let payload = id_token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
    let claims: Claims = serde_json::from_slice(&bytes).ok()?;
    const SKEW: i64 = 300;
    let valid = matches!(claims.iss.as_str(), "https://accounts.google.com" | "accounts.google.com")
        && claims.aud == client_id
        && claims.exp + SKEW > now
        && claims.iat.is_none_or(|iat| iat <= now + SKEW)
        && claims.nonce.as_deref() == Some(nonce)
        && !claims.sub.is_empty()
        && claims.sub.len() <= 255;
    valid.then_some(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encodes_reserved_characters() {
        assert_eq!(encode("openid email"), "openid%20email");
        assert_eq!(encode("https://a.b/c?d=e"), "https%3A%2F%2Fa.b%2Fc%3Fd%3De");
        assert_eq!(encode("AZaz09-._~"), "AZaz09-._~");
    }
}
