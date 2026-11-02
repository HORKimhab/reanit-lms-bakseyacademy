//! Phone normalization, password hashing and session tokens.

use argon2::{
    Argon2, PasswordHasher,
    password_hash::{
        SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Sessions last 30 days from login.
pub const SESSION_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// Normalizes a Cambodian mobile number to E.164 (`+855XXXXXXXX`).
/// Accepts `012 345 678`, `12345678`, `855 12 345 678` and `+855 12-345-678`.
pub fn normalize_phone(input: &str) -> Option<String> {
    let cleaned: String = input
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '.' | '(' | ')'))
        .collect();
    let national = if let Some(rest) = cleaned.strip_prefix("+855") {
        rest
    } else if cleaned.len() >= 11 && cleaned.starts_with("855") {
        &cleaned[3..]
    } else {
        cleaned.strip_prefix('0').unwrap_or(&cleaned)
    };
    let valid = (8..=9).contains(&national.len())
        && national.bytes().all(|b| b.is_ascii_digit())
        && !national.starts_with('0');
    valid.then(|| format!("+855{national}"))
}

pub fn validate_full_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::Validation("invalid_name"));
    }
    Ok(name.to_owned())
}

pub fn validate_password(password: &str) -> AppResult<()> {
    if !(8..=128).contains(&password.chars().count()) {
        return Err(AppError::Validation("weak_password"));
    }
    Ok(())
}

/// Argon2id hash. CPU-heavy, so it runs off the async runtime.
pub async fn hash_password(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| anyhow::anyhow!("hash password: {e}"))
    })
    .await
    .map_err(anyhow::Error::from)?
    .map_err(AppError::from)
}

/// Opaque random token for the client, and the SHA-256 we store for it.
fn new_token() -> (String, Vec<u8>) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hash_token(&token);
    (token, hash)
}

pub fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// Creates a session and returns the raw token to hand to the client.
pub async fn create_session(db: &PgPool, user_id: Uuid) -> AppResult<String> {
    let (token, hash) = new_token();
    sqlx::query(
        "INSERT INTO sessions (token_hash, user_id, expires_at) \
         VALUES ($1, $2, now() + make_interval(secs => $3))",
    )
    .bind(hash)
    .bind(user_id)
    .bind(SESSION_TTL_SECS as f64)
    .execute(db)
    .await?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_common_phone_formats() {
        for input in [
            "012 345 678",
            "12345678",
            "855 12 345 678",
            "+855 12-345-678",
            "(012) 345.678",
        ] {
            assert_eq!(
                normalize_phone(input).as_deref(),
                Some("+85512345678"),
                "{input}"
            );
        }
        assert_eq!(
            normalize_phone("097 1234 567").as_deref(),
            Some("+855971234567")
        );
    }

    #[test]
    fn rejects_invalid_phones() {
        for input in [
            "",
            "1234567",
            "0012345678",
            "+85501234567",
            "012 345 67x",
            "+1 202 555 0100",
            "1234567890",
        ] {
            assert_eq!(normalize_phone(input), None, "{input}");
        }
    }

    #[test]
    fn token_hash_is_stable_and_token_is_unique() {
        let (a, ha) = new_token();
        let (b, _) = new_token();
        assert_ne!(a, b);
        assert_eq!(hash_token(&a), ha);
        assert_eq!(a.len(), 43);
    }

    #[test]
    fn validates_name_and_password() {
        assert_eq!(validate_full_name("  សុខ ដារ៉ា ").unwrap(), "សុខ ដារ៉ា");
        assert!(validate_full_name("   ").is_err());
        assert!(validate_password("short").is_err());
        assert!(validate_password("long enough").is_ok());
    }
}
