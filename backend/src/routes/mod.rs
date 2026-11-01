mod categories;
mod courses;
mod ebooks;
mod home;

use axum::{Json, Router, extract::State, routing::get};
use serde_json::{Value, json};

use crate::{AppState, error::AppResult};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/home", get(home::home))
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
