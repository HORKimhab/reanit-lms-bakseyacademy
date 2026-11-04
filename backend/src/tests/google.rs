//! Google sign-in (server-side OAuth code flow + PKCE; ID token claims checked by the API).

use axum::http::{Method, StatusCode};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;

const NONCE: &str = "test-nonce-0123456789abcdefghijklmnopqrstuvwxyz";
const VERIFIER: &str = "verifier-0123456789abcdefghijklmnopqrstuvwxyz-ABC";

fn claims(sub: &str, email: &str) -> Value {
    let now = chrono::Utc::now().timestamp();
    json!({
        "iss": "https://accounts.google.com", "aud": GOOGLE_CLIENT_ID, "sub": sub,
        "email": email, "email_verified": true, "name": "Dara Sok",
        "iat": now, "exp": now + 3600, "nonce": NONCE
    })
}

/// Registers what Google returns for `code` (an unsigned JWT is fine: the API trusts the TLS
/// channel to Google's token endpoint, not a signature) and signs in with it.
async fn sign_in(db: &PgPool, code: &str, claims: &Value) -> Resp {
    let jwt = format!("e30.{}.c2ln", URL_SAFE_NO_PAD.encode(claims.to_string()));
    GOOGLE.0.lock().unwrap().insert(code.to_owned(), jwt);
    post(db, "/api/auth/google", None, json!({ "code": code, "code_verifier": VERIFIER, "nonce": NONCE })).await
}

async fn link(db: &PgPool, token: &str, code: &str, claims: &Value) -> Resp {
    let jwt = format!("e30.{}.c2ln", URL_SAFE_NO_PAD.encode(claims.to_string()));
    GOOGLE.0.lock().unwrap().insert(code.to_owned(), jwt);
    post(db, "/api/me/google", Some(token), json!({ "code": code, "code_verifier": VERIFIER, "nonce": NONCE })).await
}

#[sqlx::test(migrations = "./migrations")]
async fn google_builds_a_pkce_authorization_url(db: PgPool) {
    let r = post(&db, "/api/auth/google/url", None, json!({
        "state": "state-0123456789abcdefghijklmnopqrstuvwxyz-01", "nonce": NONCE,
        "code_challenge": "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    })).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let url = r.json()["url"].as_str().unwrap().to_owned();
    assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"), "{url}");
    for part in [
        &format!("client_id={GOOGLE_CLIENT_ID}"),
        "response_type=code",
        "scope=openid%20email%20profile",
        "code_challenge_method=S256",
        "code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
        "redirect_uri=https%3A%2F%2Fshop.example%2Fauth%2Fgoogle%2Fcallback",
    ] {
        assert!(url.contains(part), "{part} missing in {url}");
    }
    let r = post(&db, "/api/auth/google/url", None, json!({ "state": "x", "nonce": NONCE, "code_challenge": "y" })).await;
    assert_eq!(r.json()["error"], "invalid_oauth_params");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_new_google_user_gets_an_account_and_comes_back_to_it(db: PgPool) {
    let r = sign_in(&db, "code-1", &claims("google-sub-1", "dara@example.com")).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let user = r.json()["user"].clone();
    assert_eq!((user["email"].as_str(), user["phone"].as_str(), user["full_name"].as_str()), (Some("dara@example.com"), None, Some("Dara Sok")));
    let token = r.json()["token"].as_str().unwrap().to_owned();
    assert_eq!(get(&db, "/api/me", Some(&token)).await.json()["google_linked"], true);
    let again = sign_in(&db, "code-2", &claims("google-sub-1", "dara@example.com")).await;
    assert_eq!(again.json()["user"]["id"], user["id"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn id_token_claims_are_checked(db: PgPool) {
    let good = claims("google-sub-2", "x@example.com");
    let mut cases = Vec::new();
    for (key, value) in [
        ("aud", json!("someone-elses-client")),
        ("iss", json!("https://evil.example")),
        ("exp", json!(chrono::Utc::now().timestamp() - 600)),
        ("nonce", json!("another-nonce")),
        ("sub", json!("")),
    ] {
        let mut c = good.clone();
        c[key] = value;
        cases.push(c);
    }
    for (i, c) in cases.iter().enumerate() {
        let r = sign_in(&db, &format!("bad-{i}"), c).await;
        assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::UNAUTHORIZED, Some("google_failed")), "{c}");
    }
    // An unknown code (Google refuses it) fails the same way.
    let r = post(&db, "/api/auth/google", None, json!({ "code": "never-issued", "code_verifier": VERIFIER, "nonce": NONCE })).await;
    assert_eq!(r.json()["error"], "google_failed");
    let mut unverified = good.clone();
    unverified["email_verified"] = json!(false);
    assert_eq!(sign_in(&db, "unverified", &unverified).await.json()["error"], "google_email_unverified");
}

#[sqlx::test(migrations = "./migrations")]
async fn matching_emails_are_never_linked_automatically(db: PgPool) {
    let (user_id, token) = user(&db, "+85520000010", "student").await;
    sqlx::query("UPDATE users SET email = 'owner@example.com', password_hash = 'x' WHERE id = $1").bind(user_id).execute(&db).await.unwrap();
    let google = claims("google-sub-3", "Owner@Example.com");
    let r = sign_in(&db, "code-3", &google).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::CONFLICT, Some("google_email_in_use")));

    // Linking from the signed-in account works, and then Google signs into it.
    assert_eq!(link(&db, &token, "code-4", &google).await.status, StatusCode::NO_CONTENT);
    assert_eq!(sign_in(&db, "code-5", &google).await.json()["user"]["id"], user_id.to_string());

    // The same Google account can't be linked to a second user.
    let (_, other) = user(&db, "+85520000011", "student").await;
    let r = link(&db, &other, "code-6", &google).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::CONFLICT, Some("google_account_in_use")));
    assert_eq!(link(&db, &other, "code-7", &google).await.status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn unlinking_needs_another_way_to_sign_in(db: PgPool) {
    let r = sign_in(&db, "code-8", &claims("google-sub-4", "only@example.com")).await;
    let google_only = r.json()["token"].as_str().unwrap().to_owned();
    let r = call(&db, Method::DELETE, "/api/me/google", Some(&google_only), None).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::CONFLICT, Some("last_login_method")));

    let (user_id, token) = user(&db, "+85520000012", "student").await;
    sqlx::query("UPDATE users SET password_hash = 'x' WHERE id = $1").bind(user_id).execute(&db).await.unwrap();
    link(&db, &token, "code-9", &claims("google-sub-5", "p@example.com")).await;
    assert_eq!(call(&db, Method::DELETE, "/api/me/google", Some(&token), None).await.status, StatusCode::NO_CONTENT);
    assert_eq!(get(&db, "/api/me", Some(&token)).await.json()["google_linked"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn google_sign_in_needs_configuration(db: PgPool) {
    assert_eq!(get(&db, "/api/auth/options", None).await.json(), json!({ "google": true, "password_reset": true }));
    let none = routes::router(AppState { google: None, sms: None, ..state(db.clone()) });
    let r = call_app(none, Method::GET, "/api/auth/options", None, None).await;
    assert_eq!(r.json(), json!({ "google": false, "password_reset": false }));
    let off = routes::router(AppState { google: None, ..state(db.clone()) });
    let r = call_app(off, Method::POST, "/api/auth/google", None, Some(json!({ "code": "c", "code_verifier": VERIFIER, "nonce": NONCE }))).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::SERVICE_UNAVAILABLE, Some("google_unavailable")));
}
