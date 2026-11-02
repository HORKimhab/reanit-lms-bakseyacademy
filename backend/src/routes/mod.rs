mod auth;
mod categories;
mod courses;
mod ebooks;
mod home;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde_json::{Value, json};

use crate::{AppState, error::AppResult};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/home", get(home::home))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/me", get(auth::me))
        .route("/api/categories", get(categories::list))
        .route("/api/courses", get(courses::list))
        .route("/api/courses/{slug}", get(courses::detail))
        .route("/api/ebooks", get(ebooks::list))
        .route("/api/ebooks/{slug}", get(ebooks::detail))
        .with_state(state)
}

async fn health(State(st): State<AppState>) -> AppResult<Json<Value>> {
    sqlx::query("SELECT 1").execute(&st.db).await?;
    Ok(Json(json!({ "status": "ok" })))
}
