use std::time::Instant;

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::{
    AppState,
    auth::{self, CurrentUser, SESSION_TTL_SECS},
    error::{AppError, AppResult},
    models::{AuthResponse, User},
};

pub(crate) const USER_COLUMNS: &str = "id, full_name, phone, email, role";

#[derive(Deserialize)]
pub struct RegisterBody {
    full_name: String,
    phone: String,
    password: String,
}

pub async fn register(
    State(st): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    let full_name = auth::validate_full_name(&body.full_name)?;
    let phone = auth::normalize_phone(&body.phone).ok_or(AppError::Validation("invalid_phone"))?;
    auth::validate_password(&body.password)?;
    let password_hash = auth::hash_password(body.password).await?;

    let user = sqlx::query_as::<_, User>(&format!(
        "INSERT INTO users (full_name, phone, password_hash) VALUES ($1, $2, $3) RETURNING {USER_COLUMNS}"
    ))
    .bind(full_name)
    .bind(phone)
    .bind(password_hash)
    .fetch_one(&st.db)
    .await
    .map_err(|e| match e.as_database_error() {
        Some(db) if db.is_unique_violation() => AppError::Conflict("phone_taken"),
        _ => e.into(),
    })?;

    let token = auth::create_session(&st.db, user.id).await?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            max_age: SESSION_TTL_SECS,
            user,
        }),
    ))
}

#[derive(sqlx::FromRow)]
struct LoginRow {
    #[sqlx(flatten)]
    user: User,
    password_hash: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginBody {
    phone: String,
    password: String,
}

pub async fn login(
    State(st): State<AppState>,
    Json(body): Json<LoginBody>,
) -> AppResult<Json<AuthResponse>> {
    let phone = auth::normalize_phone(&body.phone);
    // Throttle per number (normalized when valid) against online password guessing.
    let throttle_key = phone.clone().unwrap_or_else(|| body.phone.trim().to_owned());
    if !st.login_throttle.allowed(&throttle_key, Instant::now()) {
        return Err(AppError::TooManyRequests("too_many_attempts"));
    }
    let row = match phone {
        Some(phone) => {
            sqlx::query_as::<_, LoginRow>(&format!(
                "SELECT {USER_COLUMNS}, password_hash FROM users WHERE phone = $1"
            ))
            .bind(phone)
            .fetch_optional(&st.db)
            .await?
        }
        None => None,
    };
    let (user, stored) = match row {
        Some(r) => (Some(r.user), r.password_hash),
        None => (None, None),
    };
    let user = match (auth::verify_password(body.password, stored).await?, user) {
        (true, Some(user)) => user,
        _ => {
            st.login_throttle.record_failure(&throttle_key, Instant::now());
            return Err(AppError::Unauthorized("invalid_credentials"));
        }
    };
    st.login_throttle.record_success(&throttle_key);

    sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND expires_at <= now()")
        .bind(user.id)
        .execute(&st.db)
        .await?;
    let token = auth::create_session(&st.db, user.id).await?;
    Ok(Json(AuthResponse {
        token,
        max_age: SESSION_TTL_SECS,
        user,
    }))
}

pub async fn logout(State(st): State<AppState>, current: CurrentUser) -> AppResult<StatusCode> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
        .bind(current.token_hash)
        .execute(&st.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(current: CurrentUser) -> AppResult<Json<User>> {
    Ok(Json(current.user))
}

#[derive(Deserialize)]
pub struct ProfileBody {
    full_name: String,
    email: Option<String>,
}

/// Replaces the editable profile fields. An empty `email` clears it.
pub async fn update_me(
    State(st): State<AppState>,
    current: CurrentUser,
    Json(body): Json<ProfileBody>,
) -> AppResult<Json<User>> {
    let full_name = auth::validate_full_name(&body.full_name)?;
    let email = auth::validate_email(body.email.as_deref())?;
    let user = sqlx::query_as::<_, User>(&format!(
        "UPDATE users SET full_name = $1, email = $2 WHERE id = $3 RETURNING {USER_COLUMNS}"
    ))
    .bind(full_name)
    .bind(email)
    .bind(current.user.id)
    .fetch_one(&st.db)
    .await
    .map_err(|e| match e.as_database_error() {
        Some(db) if db.is_unique_violation() => AppError::Conflict("email_taken"),
        Some(db) if db.is_check_violation() => AppError::Validation("phone_or_email_required"),
        _ => e.into(),
    })?;
    Ok(Json(user))
}

#[derive(Deserialize)]
pub struct PasswordBody {
    current_password: String,
    new_password: String,
}

/// Changes the password and signs out every other session.
pub async fn change_password(
    State(st): State<AppState>,
    current: CurrentUser,
    Json(body): Json<PasswordBody>,
) -> AppResult<StatusCode> {
    auth::validate_password(&body.new_password)?;
    let stored: Option<String> =
        sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(current.user.id)
            .fetch_one(&st.db)
            .await?;
    if !auth::verify_password(body.current_password, stored).await? {
        return Err(AppError::Validation("wrong_password"));
    }
    let hash = auth::hash_password(body.new_password).await?;

    let mut tx = st.db.begin().await?;
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(hash)
        .bind(current.user.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND token_hash <> $2")
        .bind(current.user.id)
        .bind(&current.token_hash)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
