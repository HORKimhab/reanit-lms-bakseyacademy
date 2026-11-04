mod admin;
mod admin_catalog;
mod admin_courses;
mod admin_ebooks;
mod admin_stats;
mod admin_users;
mod auth;
mod bundles;
mod cart;
mod categories;
mod certificates;
mod coupons;
mod courses;
mod ebooks;
mod home;
mod lessons;
mod library;
mod media;
mod notifications;
mod orders;
mod password_reset;
pub(crate) mod payments;
mod proofs;
mod questions;
mod recommendations;
mod reviews;
mod sitemap;
mod wishlist;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    routing::{delete, get, post, put},
};
use serde_json::{Value, json};

use crate::{AppState, error::AppResult};

/// Background confirmation of automatic payments (see `payments::spawn_poller`).
pub fn spawn_payment_poller(state: AppState) {
    payments::spawn_poller(state);
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/home", get(home::home))
        .route("/api/sitemap", get(sitemap::sitemap))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/password-reset", post(password_reset::request))
        .route("/api/auth/password-reset/confirm", post(password_reset::confirm))
        .route("/api/me", get(auth::me).patch(auth::update_me))
        .route("/api/me/password", post(auth::change_password))
        .route("/api/me/orders", get(orders::my_orders))
        .route("/api/me/library", get(library::library))
        .route("/api/me/notifications", get(notifications::list))
        .route("/api/me/wishlist", get(wishlist::list))
        .route("/api/me/wishlist/ids", get(wishlist::ids))
        .route("/api/me/wishlist/{kind}/{id}", put(wishlist::add).delete(wishlist::remove))
        .route("/api/me/notifications/read", post(notifications::read_all))
        .route("/api/recommendations", get(recommendations::recommendations))
        .route("/api/cart", get(cart::get).post(cart::add))
        .route("/api/cart/{id}", delete(cart::remove))
        .route("/api/payment-methods", get(orders::payment_methods))
        .route("/api/orders", post(orders::checkout))
        .route("/api/orders/{id}", get(orders::detail))
        .route("/api/orders/{id}/submit", post(orders::submit))
        .route("/api/orders/{id}/cancel", post(orders::cancel))
        .route("/api/orders/{id}/khqr", post(payments::khqr))
        .route("/api/orders/{id}/payway", post(payments::payway))
        .route("/api/orders/{id}/check-payment", post(payments::check_payment))
        .route("/api/payments/payway/pushback", post(payments::payway_pushback))
        .route(
            "/api/orders/{id}/proof",
            post(proofs::upload).layer(DefaultBodyLimit::max(11 * 1024 * 1024)),
        )
        .route("/api/admin/stats", get(admin_stats::stats))
        .route("/api/admin/users", get(admin_users::list))
        .route("/api/admin/users/{id}", get(admin_users::detail))
        .route("/api/admin/users/{id}/role", put(admin_users::set_role))
        .route("/api/admin/users/{id}/{kind}/{item}", put(admin_users::grant).delete(admin_users::revoke))
        .route("/api/admin/orders", get(admin::list_orders))
        .route("/api/admin/orders/{id}/approve", post(admin::approve))
        .route("/api/admin/orders/{id}/reject", post(admin::reject))
        .route(
            "/api/admin/media",
            get(media::list).post(media::upload).layer(DefaultBodyLimit::max(2100 * 1024 * 1024)),
        )
        .route("/api/telegram/webhook", post(media::webhook))
        .route("/api/media/{token}", get(media::stream))
        .route("/api/admin/coupons", get(coupons::list).post(coupons::create))
        .route("/api/admin/coupons/{id}", put(coupons::update).delete(coupons::delete))
        .route("/api/admin/categories", get(admin_catalog::list_categories).post(admin_catalog::create_category))
        .route(
            "/api/admin/categories/{id}",
            put(admin_catalog::update_category).delete(admin_catalog::delete_category),
        )
        .route("/api/admin/instructors", get(admin_catalog::list_instructors).post(admin_catalog::create_instructor))
        .route(
            "/api/admin/instructors/{id}",
            put(admin_catalog::update_instructor).delete(admin_catalog::delete_instructor),
        )
        .route("/api/admin/banners", get(admin_catalog::list_banners).post(admin_catalog::create_banner))
        .route("/api/admin/banners/{id}", put(admin_catalog::update_banner).delete(admin_catalog::delete_banner))
        .route("/api/admin/courses", get(admin_courses::list).post(admin_courses::create))
        .route(
            "/api/admin/courses/{id}",
            get(admin_courses::detail).put(admin_courses::update).delete(admin_courses::delete),
        )
        .route("/api/admin/courses/{id}/publish", post(admin_courses::publish))
        .route("/api/admin/courses/{id}/sections", post(admin_courses::create_section))
        .route(
            "/api/admin/sections/{id}",
            put(admin_courses::update_section).delete(admin_courses::delete_section),
        )
        .route("/api/admin/sections/{id}/move", post(admin_courses::move_section))
        .route("/api/admin/sections/{id}/lessons", post(admin_courses::create_lesson))
        .route("/api/admin/lessons/{id}", put(admin_courses::update_lesson).delete(admin_courses::delete_lesson))
        .route("/api/admin/lessons/{id}/move", post(admin_courses::move_lesson))
        .route("/api/admin/ebooks", get(admin_ebooks::list).post(admin_ebooks::create))
        .route(
            "/api/admin/ebooks/{id}",
            get(admin_ebooks::detail).put(admin_ebooks::update).delete(admin_ebooks::delete),
        )
        .route("/api/admin/ebooks/{id}/publish", post(admin_ebooks::publish))
        .route("/api/categories", get(categories::list))
        .route("/api/certificates/{id}", get(certificates::detail))
        .route("/api/courses", get(courses::list))
        .route("/api/courses/{slug}", get(courses::detail))
        .route("/api/courses/{slug}/review", put(reviews::put_course).delete(reviews::delete_course))
        .route("/api/courses/{slug}/lessons/{id}", get(lessons::lesson))
        .route(
            "/api/courses/{slug}/lessons/{id}/complete",
            put(lessons::complete).delete(lessons::uncomplete),
        )
        .route(
            "/api/courses/{slug}/lessons/{id}/questions",
            get(questions::list).post(questions::ask),
        )
        .route("/api/questions/{id}", delete(questions::delete_question))
        .route("/api/questions/{id}/answers", post(questions::answer))
        .route("/api/answers/{id}", delete(questions::delete_answer))
        .route("/api/admin/questions", get(questions::admin_list))
        .route("/api/bundles", get(bundles::list))
        .route("/api/bundles/{slug}", get(bundles::detail))
        .route("/api/admin/bundles", get(bundles::admin_list).post(bundles::create))
        .route(
            "/api/admin/bundles/{id}",
            get(bundles::admin_detail).put(bundles::update).delete(bundles::delete),
        )
        .route(
            "/api/admin/bundles/{id}/{kind}/{item}",
            put(bundles::add_item).delete(bundles::remove_item),
        )
        .route("/api/ebooks", get(ebooks::list))
        .route("/api/ebooks/{slug}", get(ebooks::detail))
        .route("/api/ebooks/{slug}/file", get(ebooks::file))
        .route("/api/ebooks/{slug}/review", put(reviews::put_ebook).delete(reviews::delete_ebook))
        .with_state(state)
}

async fn health(State(st): State<AppState>) -> AppResult<Json<Value>> {
    sqlx::query("SELECT 1").execute(&st.db).await?;
    Ok(Json(json!({ "status": "ok" })))
}
