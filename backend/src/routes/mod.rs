mod admin;
mod auth;
mod cart;
mod categories;
mod courses;
mod ebooks;
mod home;
mod orders;

use axum::{
    Json, Router,
    extract::State,
    routing::{delete, get, post},
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
        .route("/api/me", get(auth::me).patch(auth::update_me))
        .route("/api/me/password", post(auth::change_password))
        .route("/api/cart", get(cart::get).post(cart::add))
        .route("/api/cart/{id}", delete(cart::remove))
        .route("/api/payment-methods", get(orders::payment_methods))
        .route("/api/orders", post(orders::checkout))
        .route("/api/orders/{id}", get(orders::detail))
        .route("/api/orders/{id}/submit", post(orders::submit))
        .route("/api/orders/{id}/cancel", post(orders::cancel))
        .route("/api/admin/orders", get(admin::list_orders))
        .route("/api/admin/orders/{id}/approve", post(admin::approve))
        .route("/api/admin/orders/{id}/reject", post(admin::reject))
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
