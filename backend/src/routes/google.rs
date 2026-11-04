//! Google sign-in and account linking. Accounts are never linked by matching email (an attacker
//! could pre-register someone's address): a Google account signs into the user it is linked to,
//! or creates a new one when its verified email is unused; linking an existing account happens
//! from /account while signed in.

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};

use super::auth::USER_COLUMNS;
use crate::{
    AppState,
    auth::{self, CurrentUser, SESSION_TTL_SECS},
    error::{AppError, AppResult},
    google::{Claims, Google, verify_id_token},
    models::{AuthResponse, User},
};

/// `GET /api/auth/options`: which sign-in helpers are configured, so the login page only offers
/// what works.
pub async fn options(State(st): State<AppState>) -> Json<Value> {
    Json(json!({ "google": st.google.is_some(), "password_reset": st.sms.is_some() }))
}

fn google(st: &AppState) -> AppResult<&Google> {
    st.google.as_ref().ok_or(AppError::Unavailable("google_unavailable"))
}

/// URL-safe random values from the storefront (43–128 chars, like a PKCE verifier).
fn opaque(s: &str) -> bool {
    (43..=128).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~'))
}

#[derive(Deserialize)]
pub struct UrlBody {
    state: String,
    nonce: String,
    code_challenge: String,
}

/// `POST /api/auth/google/url` → `{url}` to send the browser to.
pub async fn url(State(st): State<AppState>, Json(body): Json<UrlBody>) -> AppResult<Json<Value>> {
    let g = google(&st)?;
    // S256 challenges are 43 base64url characters.
    let challenge_ok = body.code_challenge.len() == 43
        && body.code_challenge.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'));
    if !(opaque(&body.state) && opaque(&body.nonce) && challenge_ok) {
        return Err(AppError::Validation("invalid_oauth_params"));
    }
    Ok(Json(json!({ "url": g.authorize_url(&body.state, &body.nonce, &body.code_challenge) })))
}

#[derive(Deserialize)]
pub struct CodeBody {
    code: String,
    code_verifier: String,
    nonce: String,
}

/// Code → verified claims; 401 `google_failed` for any exchange or claim problem.
async fn claims(g: &Google, body: &CodeBody) -> AppResult<Claims> {
    let failed = || AppError::Unauthorized("google_failed");
    if body.code.is_empty() || body.code.len() > 2048 || !opaque(&body.code_verifier) || !opaque(&body.nonce) {
        return Err(failed());
    }
    let id_token = g.tokens.exchange(&body.code, &body.code_verifier, &g.redirect_uri).await.map_err(|e| {
        tracing::warn!(error = %e, "google code exchange failed");
        failed()
    })?;
    verify_id_token(&id_token, &g.client_id, &body.nonce, Utc::now().timestamp()).ok_or_else(failed)
}

/// `POST /api/auth/google`: signs in the linked user, or creates one. 409 `google_email_in_use`
/// (an account already uses that email: sign in and link from /account), 422
/// `google_email_unverified` (can't create an account from an unverified address).
pub async fn sign_in(State(st): State<AppState>, Json(body): Json<CodeBody>) -> AppResult<Json<AuthResponse>> {
    let g = google(&st)?;
    let claims = claims(g, &body).await?;
    let linked = sqlx::query_as::<_, User>(&format!("SELECT {USER_COLUMNS} FROM users WHERE google_sub = $1"))
        .bind(&claims.sub)
        .fetch_optional(&st.db)
        .await?;
    let user = match linked {
        Some(user) => user,
        None => {
            let email = match (claims.email_verified, claims.email.as_deref()) {
                (true, Some(email)) => auth::validate_email(Some(email))?.ok_or(AppError::Validation("google_email_unverified"))?,
                _ => return Err(AppError::Validation("google_email_unverified")),
            };
            let name = claims
                .name
                .as_deref()
                .and_then(|n| auth::validate_full_name(n).ok())
                .unwrap_or_else(|| email.split('@').next().unwrap_or("Google").to_owned());
            sqlx::query_as::<_, User>(&format!(
                "INSERT INTO users (full_name, email, google_sub) VALUES ($1, $2, $3) RETURNING {USER_COLUMNS}"
            ))
            .bind(name)
            .bind(&email)
            .bind(&claims.sub)
            .fetch_one(&st.db)
            .await
            .map_err(|e| match e.as_database_error() {
                // lower(email) or google_sub already taken (a parallel sign-in): never merge.
                Some(db) if db.is_unique_violation() => AppError::Conflict("google_email_in_use"),
                _ => e.into(),
            })?
        }
    };
    let token = auth::create_session(&st.db, user.id).await?;
    Ok(Json(AuthResponse { token, max_age: SESSION_TTL_SECS, user }))
}

/// `POST /api/me/google`: links the Google account to the signed-in user. 409
/// `google_account_in_use` (linked to someone else) / `google_already_linked` (this user has another).
pub async fn link(State(st): State<AppState>, current: CurrentUser, Json(body): Json<CodeBody>) -> AppResult<StatusCode> {
    let g = google(&st)?;
    let claims = claims(g, &body).await?;
    let owner: Option<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE google_sub = $1")
        .bind(&claims.sub)
        .fetch_optional(&st.db)
        .await?;
    match owner {
        Some(id) if id == current.user.id => return Ok(StatusCode::NO_CONTENT),
        Some(_) => return Err(AppError::Conflict("google_account_in_use")),
        None => {}
    }
    let updated = sqlx::query("UPDATE users SET google_sub = $2 WHERE id = $1 AND google_sub IS NULL")
        .bind(current.user.id)
        .bind(&claims.sub)
        .execute(&st.db)
        .await
        .map_err(|e| match e.as_database_error() {
            Some(db) if db.is_unique_violation() => AppError::Conflict("google_account_in_use"),
            _ => e.into(),
        })?
        .rows_affected();
    if updated == 0 {
        return Err(AppError::Conflict("google_already_linked"));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/me/google`: unlinks, unless Google is the only way in (409 `last_login_method`).
pub async fn unlink(State(st): State<AppState>, current: CurrentUser) -> AppResult<StatusCode> {
    let has_password: bool = sqlx::query_scalar("SELECT password_hash IS NOT NULL FROM users WHERE id = $1")
        .bind(current.user.id)
        .fetch_one(&st.db)
        .await?;
    if !has_password {
        return Err(AppError::Conflict("last_login_method"));
    }
    sqlx::query("UPDATE users SET google_sub = NULL WHERE id = $1").bind(current.user.id).execute(&st.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
