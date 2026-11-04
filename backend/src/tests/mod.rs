//! Integration tests: the real router against a throwaway database per test (`#[sqlx::test]`
//! creates it on the server in `DATABASE_URL` and applies `migrations/`).

mod access;
mod admin_users;
mod bundles;
mod certificates;
mod coupons;
mod orders;
mod payment_flow;
mod questions;
mod wishlist;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use futures_util::future::BoxFuture;

use crate::{
    AppState, auth,
    payments::{self, Gateway, Lookup, Payments, khqr, payway},
    routes,
    telegram::Telegram,
    video::BunnyStream,
};

/// The secrets these tests check never leak (see the access-control invariants in `CLAUDE.md`).
pub const PREVIEW_VIDEO: &str = "vid-preview-0001";
pub const PAID_VIDEO: &str = "vid-paid-secret-0002";
pub const FILE_KEY: &str = "private/ebook-secret-key.pdf";
pub const TG_FILE_ID: &str = "tg-file-id-secret-0003";
pub const TG_UNIQUE_ID: &str = "tg-unique-secret-0004";
pub const BOT_TOKEN: &str = "123456:bot-token-secret";

/// Stands in for Bakong / PayWay. Payment refs (KHQR md5, PayWay tran_id) are unique, so one
/// shared fake serves every test; unknown refs are `Pending`.
#[derive(Default)]
pub struct FakeGateway(Mutex<HashMap<String, Lookup>>);

impl Gateway for FakeGateway {
    fn lookup<'a>(&'a self, provider_ref: &'a str) -> BoxFuture<'a, anyhow::Result<Lookup>> {
        let found = self.0.lock().unwrap().get(provider_ref).cloned().unwrap_or(Lookup::Pending);
        Box::pin(async move { Ok(found) })
    }
}

pub static GATEWAY: LazyLock<Arc<FakeGateway>> = LazyLock::new(Default::default);

/// What the fake provider reports for `provider_ref` from now on.
pub fn provider_reports(provider_ref: &str, lookup: Lookup) {
    GATEWAY.0.lock().unwrap().insert(provider_ref.to_owned(), lookup);
}

pub const BAKONG_ACCOUNT: &str = "reanit@aclb";
pub const SITE_URL: &str = "https://shop.example";

pub fn test_payments() -> Payments {
    Payments {
        bakong: Some(payments::Bakong {
            merchant: khqr::Merchant {
                account_id: BAKONG_ACCOUNT.into(),
                name: "Reanit Academy".into(),
                city: "Phnom Penh".into(),
                merchant_id: None,
                acquiring_bank: None,
            },
            gateway: GATEWAY.clone(),
        }),
        payway: Some(payments::Payway {
            config: payway::Config {
                merchant_id: "ec000262".into(),
                api_key: "test-api-key".into(),
                api_url: "https://checkout-sandbox.payway.com.kh".into(),
            },
            gateway: GATEWAY.clone(),
            site_url: SITE_URL.into(),
        }),
    }
}

pub fn state(db: PgPool) -> AppState {
    AppState {
        db,
        video: Some(BunnyStream { library_id: "lib".into(), token_key: Some("bunny-key".into()) }),
        storage: None,
        // Never contacted: the routes under test only sign `/media/…` links.
        telegram: Some(Telegram::new("http://127.0.0.1:9", BOT_TOKEN.into(), "-100".into(), None)),
        media_key: (*b"test-media-signing-key-32-bytes!").into(),
        login_throttle: Default::default(),
        payments: test_payments(),
    }
}

pub fn app(db: &PgPool) -> Router {
    routes::router(state(db.clone()))
}

pub struct Resp {
    pub status: StatusCode,
    pub text: String,
}

impl Resp {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.text).unwrap_or(Value::Null)
    }

    /// Fails if any of the secrets appears anywhere in the body.
    pub fn assert_no_secrets(&self, secrets: &[&str]) {
        for s in secrets {
            assert!(!self.text.contains(s), "response leaks {s}: {}", self.text);
        }
    }
}

pub async fn call(db: &PgPool, method: Method, path: &str, token: Option<&str>, body: Option<Value>) -> Resp {
    call_app(app(db), method, path, token, body).await
}

/// Like [`call`], against a router built from a custom state.
pub async fn call_app(app: Router, method: Method, path: &str, token: Option<&str>, body: Option<Value>) -> Resp {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    let req = match body {
        Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let res = app.oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    Resp { status, text: String::from_utf8_lossy(&bytes).into_owned() }
}

pub async fn get(db: &PgPool, path: &str, token: Option<&str>) -> Resp {
    call(db, Method::GET, path, token, None).await
}

pub async fn post(db: &PgPool, path: &str, token: Option<&str>, body: Value) -> Resp {
    call(db, Method::POST, path, token, Some(body)).await
}

/// A user with a session; returns `(id, token)`.
pub async fn user(db: &PgPool, phone: &str, role: &str) -> (Uuid, String) {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (full_name, phone, role) VALUES ('Test', $1, $2) RETURNING id",
    )
    .bind(phone)
    .bind(role)
    .fetch_one(db)
    .await
    .unwrap();
    (id, auth::create_session(db, id).await.unwrap())
}

pub struct Catalog {
    pub course_id: Uuid,
    pub preview_lesson: Uuid,
    pub paid_lesson: Uuid,
    pub tg_lesson: Uuid,
    pub ebook_id: Uuid,
}

/// One published course (`demo-course`: a preview lesson, a paid Bunny lesson, a paid Telegram
/// lesson), one published ebook (`demo-ebook`, S3 key) and an active payment method `khqr`.
pub async fn catalog(db: &PgPool) -> Catalog {
    let instructor: Uuid = sqlx::query_scalar("INSERT INTO instructors (name) VALUES ('Teacher') RETURNING id")
        .fetch_one(db)
        .await
        .unwrap();
    let course_id: Uuid = sqlx::query_scalar(
        "INSERT INTO courses (slug, title, instructor_id, price_cents, published) \
         VALUES ('demo-course', 'Demo course', $1, 1000, true) RETURNING id",
    )
    .bind(instructor)
    .fetch_one(db)
    .await
    .unwrap();
    let section: Uuid = sqlx::query_scalar(
        "INSERT INTO course_sections (course_id, title, position) VALUES ($1, 'Intro', 1) RETURNING id",
    )
    .bind(course_id)
    .fetch_one(db)
    .await
    .unwrap();
    let media: Uuid = sqlx::query_scalar(
        "INSERT INTO media_files (provider, telegram_file_id, telegram_file_unique_id, telegram_message_id, mime_type) \
         VALUES ('telegram', $1, $2, 42, 'video/mp4') RETURNING id",
    )
    .bind(TG_FILE_ID)
    .bind(TG_UNIQUE_ID)
    .fetch_one(db)
    .await
    .unwrap();
    let lesson = |pos: i32, title: &'static str, video: Option<&'static str>, preview: bool, media: Option<Uuid>| {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO lessons (section_id, title, position, video_id, duration_seconds, is_preview, media_id) \
             VALUES ($1, $2, $3, $4, 60, $5, $6) RETURNING id",
        )
        .bind(section)
        .bind(title)
        .bind(pos)
        .bind(video)
        .bind(preview)
        .bind(media)
        .fetch_one(db)
    };
    let preview_lesson = lesson(1, "Preview", Some(PREVIEW_VIDEO), true, None).await.unwrap();
    let paid_lesson = lesson(2, "Paid", Some(PAID_VIDEO), false, None).await.unwrap();
    let tg_lesson = lesson(3, "Telegram", None, false, Some(media)).await.unwrap();
    let ebook_id: Uuid = sqlx::query_scalar(
        "INSERT INTO ebooks (slug, title, author_name, file_key, price_cents, published) \
         VALUES ('demo-ebook', 'Demo ebook', 'Author', $1, 500, true) RETURNING id",
    )
    .bind(FILE_KEY)
    .fetch_one(db)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO payment_methods (code, name_km, name_en, active) VALUES ('khqr', 'KHQR', 'KHQR', true), \
         ('bakong_khqr', 'Bakong', 'Bakong', true), ('aba_payway', 'PayWay', 'PayWay', true)",
    )
    .execute(db)
    .await
    .unwrap();
    Catalog { course_id, preview_lesson, paid_lesson, tg_lesson, ebook_id }
}

/// Puts both catalog items in the user's cart and checks out; returns the order id.
pub async fn order_both(db: &PgPool, token: &str, c: &Catalog) -> Uuid {
    order_with(db, token, c, "khqr").await
}

/// [`order_both`] paying with `method`.
pub async fn order_with(db: &PgPool, token: &str, c: &Catalog, method: &str) -> Uuid {
    let r = post(db, "/api/cart", Some(token), serde_json::json!({ "course_id": c.course_id })).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let r = post(db, "/api/cart", Some(token), serde_json::json!({ "ebook_id": c.ebook_id })).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let r = post(db, "/api/orders", Some(token), serde_json::json!({ "payment_method": method })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.text);
    r.json()["id"].as_str().unwrap().parse().unwrap()
}

/// Grants the course and ebook directly (as an approved order would).
pub async fn grant_both(db: &PgPool, user_id: Uuid, c: &Catalog) {
    sqlx::query("INSERT INTO enrollments (user_id, course_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(c.course_id)
        .execute(db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO ebook_purchases (user_id, ebook_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(c.ebook_id)
        .execute(db)
        .await
        .unwrap();
}
