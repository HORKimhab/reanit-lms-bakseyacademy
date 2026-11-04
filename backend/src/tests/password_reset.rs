//! Self-service password reset by SMS code.

use std::time::Duration;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::*;

/// The newest 6-digit code texted to `phone` (sending happens in the background).
async fn texted_code(phone: &str) -> Option<String> {
    for _ in 0..100 {
        if let Some(text) = SMS.0.lock().unwrap().get(phone).and_then(|t| t.last().cloned()) {
            let digits: Vec<&str> = text.split(|c: char| !c.is_ascii_digit()).filter(|d| d.len() == 6).collect();
            return digits.first().map(|d| (*d).to_owned());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    None
}

fn texts(phone: &str) -> usize {
    SMS.0.lock().unwrap().get(phone).map_or(0, Vec::len)
}

async fn request(db: &PgPool, phone: &str) -> Resp {
    post(db, "/api/auth/password-reset", None, json!({ "phone": phone })).await
}

async fn confirm(db: &PgPool, phone: &str, code: &str, password: &str) -> Resp {
    post(db, "/api/auth/password-reset/confirm", None, json!({ "phone": phone, "code": code, "new_password": password })).await
}

async fn set_password(db: &PgPool, phone: &str) {
    let hash = crate::auth::hash_password("old password 1".into()).await.unwrap();
    sqlx::query("UPDATE users SET password_hash = $2 WHERE phone = $1").bind(phone).bind(hash).execute(db).await.unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn a_texted_code_resets_the_password_once(db: PgPool) {
    let phone = "+85520000001";
    let (_, old_session) = user(&db, phone, "student").await;
    set_password(&db, phone).await;

    let r = request(&db, "020 000 001").await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "{}", r.text);
    let code = texted_code(phone).await.expect("code texted");
    let r = confirm(&db, "020000001", &code, "short").await;
    assert_eq!(r.json()["error"], "weak_password");

    let r = confirm(&db, phone, &code, "brand new password").await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let token = r.json()["token"].as_str().unwrap().to_owned();
    assert_eq!(get(&db, "/api/me", Some(&token)).await.status, StatusCode::OK);
    // Every other session is signed out.
    assert_eq!(get(&db, "/api/me", Some(&old_session)).await.status, StatusCode::UNAUTHORIZED);
    let login = post(&db, "/api/auth/login", None, json!({ "phone": phone, "password": "brand new password" })).await;
    assert_eq!(login.status, StatusCode::OK);
    // A code works once.
    assert_eq!(confirm(&db, phone, &code, "another password").await.json()["error"], "invalid_code");
}

#[sqlx::test(migrations = "./migrations")]
async fn unknown_numbers_look_the_same_and_get_no_text(db: PgPool) {
    let phone = "+85520000002";
    let r = request(&db, phone).await;
    assert_eq!(r.status, StatusCode::ACCEPTED);
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(texts(phone), 0);
    assert_eq!(confirm(&db, phone, "123456", "brand new password").await.json()["error"], "invalid_code");
    assert_eq!(request(&db, "not a phone").await.json()["error"], "invalid_phone");
}

#[sqlx::test(migrations = "./migrations")]
async fn wrong_guesses_burn_the_code(db: PgPool) {
    let phone = "+85520000003";
    user(&db, phone, "student").await;
    request(&db, phone).await;
    let code = texted_code(phone).await.unwrap();
    let wrong = if code == "000000" { "111111" } else { "000000" };
    for _ in 0..5 {
        assert_eq!(confirm(&db, phone, wrong, "brand new password").await.json()["error"], "invalid_code");
    }
    assert_eq!(confirm(&db, phone, &code, "brand new password").await.json()["error"], "invalid_code");
}

#[sqlx::test(migrations = "./migrations")]
async fn codes_expire_and_a_new_request_replaces_the_old_code(db: PgPool) {
    let phone = "+85520000004";
    user(&db, phone, "student").await;
    request(&db, phone).await;
    let first = texted_code(phone).await.unwrap();
    SMS.0.lock().unwrap().remove(phone);
    request(&db, phone).await;
    let second = texted_code(phone).await.unwrap();
    if first != second {
        assert_eq!(confirm(&db, phone, &first, "brand new password").await.json()["error"], "invalid_code");
    }
    // Ten minutes later the code is useless.
    sqlx::query("UPDATE password_resets SET expires_at = now() - interval '1 second'").execute(&db).await.unwrap();
    assert_eq!(confirm(&db, phone, &second, "brand new password").await.json()["error"], "invalid_code");
}

#[sqlx::test(migrations = "./migrations")]
async fn requests_are_rate_limited_per_number(db: PgPool) {
    // Applies whether or not the number has an account, so it reveals nothing.
    for phone in ["+85520000005", "+85520000006"] {
        if phone.ends_with('5') {
            user(&db, phone, "student").await;
        }
        for _ in 0..3 {
            assert_eq!(request(&db, phone).await.status, StatusCode::ACCEPTED);
        }
        let r = request(&db, phone).await;
        assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::TOO_MANY_REQUESTS, Some("too_many_attempts")));
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn reset_needs_an_sms_provider(db: PgPool) {
    let no_sms = routes::router(AppState { sms: None, ..state(db.clone()) });
    let r = call_app(no_sms, Method::POST, "/api/auth/password-reset", None, Some(json!({ "phone": "+85520000007" }))).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::SERVICE_UNAVAILABLE, Some("sms_unavailable")));
}
