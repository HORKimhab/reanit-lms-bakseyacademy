//! Self-service password reset with a 6-digit SMS code.
//!
//! Requesting a code always answers 202 (the same for unknown numbers, and the text goes out in
//! the background), is limited to 3 per number per hour, and only Cambodian numbers are accepted
//! (`normalize_phone`), which bounds SMS cost. Codes are stored as an HMAC, expire after 10
//! minutes, die after 5 wrong guesses and work once; using one signs out every other session.

use std::time::Instant;

use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::{Json, extract::State, http::StatusCode};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use uuid::Uuid;

use super::auth::USER_COLUMNS;
use crate::{
    AppState,
    auth::{self, SESSION_TTL_SECS},
    error::{AppError, AppResult},
    models::{AuthResponse, User},
};

const CODE_TTL_MINUTES: i32 = 10;
const MAX_GUESSES: i32 = 5;

/// Uniform 6-digit code (rejection sampling avoids modulo bias).
fn new_code() -> String {
    loop {
        let n = OsRng.next_u32();
        if n < 4_294_000_000 {
            return format!("{:06}", n % 1_000_000);
        }
    }
}

/// HMAC of the code, bound to the user, keyed with the server secret: a leaked table can't be
/// brute-forced offline without the key.
fn code_mac(key: &[u8], user_id: Uuid, code: &str) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC takes any key length");
    mac.update(b"password-reset:");
    mac.update(user_id.as_bytes());
    mac.update(code.as_bytes());
    mac
}

#[derive(Deserialize)]
pub struct RequestBody {
    phone: String,
}

/// `POST /api/auth/password-reset`: texts a code if the number has an account. 202 either way;
/// 422 `invalid_phone`, 429 `too_many_attempts`, 503 `sms_unavailable`.
pub async fn request(State(st): State<AppState>, Json(body): Json<RequestBody>) -> AppResult<StatusCode> {
    let sms = st.sms.clone().ok_or(AppError::Unavailable("sms_unavailable"))?;
    let phone = auth::normalize_phone(&body.phone).ok_or(AppError::Validation("invalid_phone"))?;
    let now = Instant::now();
    if !st.reset_throttle.0.allowed(&phone, now) {
        return Err(AppError::TooManyRequests("too_many_attempts"));
    }
    st.reset_throttle.0.record_failure(&phone, now);

    let user_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE phone = $1")
        .bind(&phone)
        .fetch_optional(&st.db)
        .await?;
    let Some(user_id) = user_id else { return Ok(StatusCode::ACCEPTED) };
    let code = new_code();
    let mut tx = st.db.begin().await?;
    // Only the newest code counts.
    sqlx::query("UPDATE password_resets SET used_at = now() WHERE user_id = $1 AND used_at IS NULL")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO password_resets (user_id, code_hash, expires_at) VALUES ($1, $2, now() + make_interval(mins => $3))",
    )
    .bind(user_id)
    .bind(code_mac(&st.media_key, user_id, &code).finalize().into_bytes().to_vec())
    .bind(CODE_TTL_MINUTES)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let text = format!(
        "Reanit Academy: លេខកូដប្តូរពាក្យសម្ងាត់ {code} (ផុតកំណត់ ១០ នាទី)។ កុំប្រាប់អ្នកណា។ / Password reset code {code}. Don't share it."
    );
    tokio::spawn(async move {
        if let Err(e) = sms.send(&phone, &text).await {
            tracing::warn!(error = %e, "password reset SMS failed");
        }
    });
    Ok(StatusCode::ACCEPTED)
}

#[derive(Deserialize)]
pub struct ConfirmBody {
    phone: String,
    code: String,
    new_password: String,
}

#[derive(sqlx::FromRow)]
struct ActiveReset {
    id: Uuid,
    user_id: Uuid,
    code_hash: Vec<u8>,
}

/// `POST /api/auth/password-reset/confirm`: sets the new password and signs in (`{token, max_age,
/// user}`), signing out every other session. 422 `invalid_code` (wrong, expired, used, burned or no
/// such number, all alike) / `weak_password`.
pub async fn confirm(State(st): State<AppState>, Json(body): Json<ConfirmBody>) -> AppResult<Json<AuthResponse>> {
    auth::validate_password(&body.new_password)?;
    let invalid = || AppError::Validation("invalid_code");
    let phone = auth::normalize_phone(&body.phone).ok_or_else(invalid)?;
    let code = body.code.trim();
    let reset = sqlx::query_as::<_, ActiveReset>(
        "SELECT r.id, r.user_id, r.code_hash FROM password_resets r JOIN users u ON u.id = r.user_id
         WHERE u.phone = $1 AND r.used_at IS NULL AND r.expires_at > now() AND r.attempts < $2
         ORDER BY r.created_at DESC LIMIT 1",
    )
    .bind(&phone)
    .bind(MAX_GUESSES)
    .fetch_optional(&st.db)
    .await?
    .ok_or_else(invalid)?;
    // Count the guess before checking it, so parallel guesses can't exceed the limit.
    let counted = sqlx::query("UPDATE password_resets SET attempts = attempts + 1 WHERE id = $1 AND attempts < $2")
        .bind(reset.id)
        .bind(MAX_GUESSES)
        .execute(&st.db)
        .await?
        .rows_affected();
    let well_formed = code.len() == 6 && code.bytes().all(|b| b.is_ascii_digit());
    if counted == 0 || !well_formed || code_mac(&st.media_key, reset.user_id, code).verify_slice(&reset.code_hash).is_err() {
        return Err(invalid());
    }

    let password_hash = auth::hash_password(body.new_password).await?;
    let mut tx = st.db.begin().await?;
    let used = sqlx::query("UPDATE password_resets SET used_at = now() WHERE id = $1 AND used_at IS NULL")
        .bind(reset.id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if used == 0 {
        return Err(invalid());
    }
    let user = sqlx::query_as::<_, User>(&format!(
        "UPDATE users SET password_hash = $2 WHERE id = $1 RETURNING {USER_COLUMNS}"
    ))
    .bind(reset.user_id)
    .bind(password_hash)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1").bind(reset.user_id).execute(&mut *tx).await?;
    tx.commit().await?;
    st.login_throttle.record_success(&phone);
    let token = auth::create_session(&st.db, user.id).await?;
    Ok(Json(AuthResponse { token, max_age: SESSION_TTL_SECS, user }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_six_digits_and_vary() {
        let codes: std::collections::HashSet<String> = (0..50).map(|_| new_code()).collect();
        assert!(codes.iter().all(|c| c.len() == 6 && c.bytes().all(|b| b.is_ascii_digit())));
        assert!(codes.len() > 40);
    }
}
