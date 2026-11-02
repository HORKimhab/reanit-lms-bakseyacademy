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
    let row = match auth::normalize_phone(&body.phone) {
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
        _ => return Err(AppError::Unauthorized("invalid_credentials")),
    };

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
