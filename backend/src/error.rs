use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    /// Request data failed validation; the code tells the client which rule.
    #[error("validation: {0}")]
    Validation(&'static str),
    /// The request conflicts with existing data (e.g. `phone_taken`).
    #[error("conflict: {0}")]
    Conflict(&'static str),
    /// Missing/expired session (`unauthorized`) or failed login (`invalid_credentials`).
    #[error("unauthorized: {0}")]
    Unauthorized(&'static str),
    /// Rate limited (e.g. `too_many_attempts` after repeated wrong passwords).
    #[error("too many requests: {0}")]
    TooManyRequests(&'static str),
    /// Signed in but not allowed (e.g. not an admin).
    #[error("forbidden")]
    Forbidden,
    /// A dependency the request needs isn't configured or reachable (e.g. `storage_unavailable`).
    #[error("unavailable: {0}")]
    Unavailable(&'static str),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            AppError::Validation(code) => (StatusCode::UNPROCESSABLE_ENTITY, *code),
            AppError::Conflict(code) => (StatusCode::CONFLICT, *code),
            AppError::Unauthorized(code) => (StatusCode::UNAUTHORIZED, *code),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            AppError::TooManyRequests(code) => (StatusCode::TOO_MANY_REQUESTS, *code),
            AppError::Unavailable(code) => (StatusCode::SERVICE_UNAVAILABLE, *code),
            AppError::Db(e) => {
                tracing::error!(error = %e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
            AppError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };
        (status, Json(json!({ "error": code }))).into_response()
    }
}
