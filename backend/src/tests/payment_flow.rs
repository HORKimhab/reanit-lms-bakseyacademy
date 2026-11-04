//! Automatic payment confirmation (Bakong KHQR, ABA PayWay) against a fake provider: an order is
//! paid and granted exactly once, and only when the provider confirms the right amount to us.

use axum::http::{Method, StatusCode};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use super::*;

async fn grants(db: &PgPool, user_id: Uuid) -> (i64, i64) {
    sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM enrollments WHERE user_id = $1), (SELECT COUNT(*) FROM ebook_purchases WHERE user_id = $1)",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .unwrap()
}

fn paid(cents: i64, currency: &str, to: Option<&str>, txn: &str) -> Lookup {
    Lookup::Paid { amount_cents: cents, currency: currency.into(), to_account: to.map(Into::into), provider_txn: txn.into() }
}

/// The KHQR string for the order, and the md5 Bakong knows it by.
async fn khqr(db: &PgPool, token: &str, order: Uuid) -> (String, String) {
    let r = post(db, &format!("/api/orders/{order}/khqr"), Some(token), json!({})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let qr = r.json()["qr"].as_str().unwrap().to_owned();
    let md5 = khqr::md5_hex(&qr);
    (qr, md5)
}

async fn status(db: &PgPool, token: &str, order: Uuid) -> Value {
    let r = post(db, &format!("/api/orders/{order}/check-payment"), Some(token), json!({})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    r.json()
}

#[sqlx::test(migrations = "./migrations")]
async fn bakong_payment_is_confirmed_once(db: PgPool) {
    let c = catalog(&db).await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;
    let (_, stranger) = user(&db, "+85510000002", "student").await;
    let methods = get(&db, "/api/payment-methods", None).await.json();
    let automatic = |code: &str| methods.as_array().unwrap().iter().find(|m| m["code"] == code).unwrap()["automatic"].clone();
    assert_eq!((automatic("bakong_khqr"), automatic("aba_payway"), automatic("khqr")), (json!(true), json!(true), json!(false)));

    let order = order_with(&db, &buyer, &c, "bakong_khqr").await;
    assert_eq!(post(&db, &format!("/api/orders/{order}/khqr"), None, json!({})).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(post(&db, &format!("/api/orders/{order}/khqr"), Some(&stranger), json!({})).await.status, StatusCode::NOT_FOUND);
    let (qr, md5) = khqr(&db, &buyer, order).await;
    assert!(qr.starts_with("000201010212") && qr.contains("540515.00") && qr.contains(BAKONG_ACCOUNT), "{qr}");
    assert_eq!(khqr::crc16(&qr[..qr.len() - 4]), qr[qr.len() - 4..]);
    // Asking again shows the same (still valid) QR.
    assert_eq!(khqr(&db, &buyer, order).await.0, qr);

    assert_eq!(status(&db, &buyer, order).await["status"], "pending_payment");
    assert_eq!(grants(&db, buyer_id).await, (0, 0));

    provider_reports(&md5, paid(1500, "USD", Some(BAKONG_ACCOUNT), "bakong-hash-1"));
    let check = format!("/api/orders/{order}/check-payment");
    let (a, b) = tokio::join!(post(&db, &check, Some(&buyer), json!({})), post(&db, &check, Some(&buyer), json!({})));
    assert_eq!((a.status, b.status), (StatusCode::OK, StatusCode::OK));
    let o = status(&db, &buyer, order).await;
    assert_eq!((o["status"].as_str(), o["paid_via"].as_str(), o["payment_ref"].as_str()), (Some("paid"), Some("bakong"), Some("bakong-hash-1")));
    assert_eq!(grants(&db, buyer_id).await, (1, 1));
    let n = get(&db, "/api/me/notifications", Some(&buyer)).await.json();
    assert_eq!((n["unread_count"].as_i64(), n["items"][0]["kind"].as_str()), (Some(1), Some("order_paid")));
    // A paid order has nothing left to pay.
    let r = post(&db, &format!("/api/orders/{order}/khqr"), Some(&buyer), json!({})).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::CONFLICT, Some("invalid_status")));
    // Other people can't poll it.
    assert_eq!(post(&db, &format!("/api/orders/{order}/check-payment"), Some(&stranger), json!({})).await.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn payments_that_dont_match_are_not_accepted(db: PgPool) {
    let c = catalog(&db).await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;
    let order = order_with(&db, &buyer, &c, "bakong_khqr").await;
    for wrong in [
        paid(1400, "USD", Some(BAKONG_ACCOUNT), "short"),
        paid(1500, "KHR", Some(BAKONG_ACCOUNT), "khr"),
        paid(1500, "USD", Some("someone@else"), "elsewhere"),
    ] {
        let (_, md5) = khqr(&db, &buyer, order).await;
        provider_reports(&md5, wrong);
        assert_eq!(status(&db, &buyer, order).await["status"], "pending_payment");
    }
    assert_eq!(grants(&db, buyer_id).await, (0, 0));
    // Each rejected attempt is closed, so the buyer gets a fresh QR; paying that one works.
    let (_, md5) = khqr(&db, &buyer, order).await;
    provider_reports(&md5, paid(1500, "USD", Some(BAKONG_ACCOUNT), "good"));
    assert_eq!(status(&db, &buyer, order).await["status"], "paid");
}

#[sqlx::test(migrations = "./migrations")]
async fn providers_need_the_matching_method_and_configuration(db: PgPool) {
    let c = catalog(&db).await;
    let (_, buyer) = user(&db, "+85510000001", "student").await;
    let manual = order_with(&db, &buyer, &c, "khqr").await;
    let r = post(&db, &format!("/api/orders/{manual}/khqr"), Some(&buyer), json!({})).await;
    assert_eq!(r.json()["error"], "invalid_payment_method");
    let r = post(&db, &format!("/api/orders/{manual}/payway"), Some(&buyer), json!({})).await;
    assert_eq!(r.json()["error"], "invalid_payment_method");
    // Manual orders still work as before.
    let r = post(&db, &format!("/api/orders/{manual}/submit"), Some(&buyer), json!({ "payment_ref": "TX-1" })).await;
    assert_eq!(r.json()["status"], "awaiting_approval");

    let (_, other) = user(&db, "+85510000002", "student").await;
    let order = order_with(&db, &other, &c, "bakong_khqr").await;
    let unconfigured = routes::router(AppState { payments: Payments::default(), ..state(db.clone()) });
    let r = call_app(unconfigured, Method::POST, &format!("/api/orders/{order}/khqr"), Some(&other), Some(json!({}))).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::SERVICE_UNAVAILABLE, Some("payment_unavailable")));
}

#[sqlx::test(migrations = "./migrations")]
async fn payway_pushback_is_verified_with_payway(db: PgPool) {
    let c = catalog(&db).await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;
    let order = order_with(&db, &buyer, &c, "aba_payway").await;
    let r = post(&db, &format!("/api/orders/{order}/payway"), Some(&buyer), json!({})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let checkout = r.json();
    assert_eq!(checkout["action_url"], "https://checkout-sandbox.payway.com.kh/api/payment-gateway/v1/payments/purchase");
    let field = |k: &str| {
        checkout["fields"].as_array().unwrap().iter().find(|f| f[0] == k).map(|f| f[1].as_str().unwrap().to_owned())
    };
    let tran_id = field("tran_id").unwrap();
    assert!(tran_id.len() <= 20 && tran_id.len() >= 16, "{tran_id}");
    assert_eq!(field("amount").as_deref(), Some("15.00"));
    assert!(field("hash").is_some());
    let return_url = String::from_utf8(B64.decode(field("return_url").unwrap()).unwrap()).unwrap();
    assert_eq!(return_url, format!("{SITE_URL}/payments/payway/pushback"));

    let pushback = |body: Value| post(&db, "/api/payments/payway/pushback", None, body);
    assert_eq!(pushback(json!({ "tran_id": "unknown-tran" })).await.status, StatusCode::NOT_FOUND);
    // A pushback claiming success proves nothing until PayWay confirms it.
    assert_eq!(pushback(json!({ "tran_id": tran_id, "status": "00", "apv": "123" })).await.status, StatusCode::OK);
    assert_eq!(status(&db, &buyer, order).await["status"], "pending_payment");

    provider_reports(&tran_id, paid(1500, "USD", None, "apv-753786"));
    for _ in 0..2 {
        assert_eq!(pushback(json!({ "tran_id": tran_id, "status": "00" })).await.status, StatusCode::OK);
    }
    let o = status(&db, &buyer, order).await;
    assert_eq!((o["status"].as_str(), o["paid_via"].as_str()), (Some("paid"), Some("payway")));
    assert_eq!(grants(&db, buyer_id).await, (1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn declined_payway_attempts_leave_the_order_payable(db: PgPool) {
    let c = catalog(&db).await;
    let (_, buyer) = user(&db, "+85510000001", "student").await;
    let order = order_with(&db, &buyer, &c, "aba_payway").await;
    let first = post(&db, &format!("/api/orders/{order}/payway"), Some(&buyer), json!({})).await.json();
    let tran_id = first["fields"].as_array().unwrap().iter().find(|f| f[0] == "tran_id").unwrap()[1].as_str().unwrap().to_owned();
    provider_reports(&tran_id, Lookup::Failed);
    assert_eq!(status(&db, &buyer, order).await["status"], "pending_payment");
    // Every attempt gets its own tran_id.
    let second = post(&db, &format!("/api/orders/{order}/payway"), Some(&buyer), json!({})).await.json();
    assert_ne!(second["fields"], first["fields"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_payment_made_before_cancelling_wins(db: PgPool) {
    let c = catalog(&db).await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;
    let order = order_with(&db, &buyer, &c, "bakong_khqr").await;
    let (_, md5) = khqr(&db, &buyer, order).await;
    provider_reports(&md5, paid(1500, "USD", Some(BAKONG_ACCOUNT), "late"));
    let r = post(&db, &format!("/api/orders/{order}/cancel"), Some(&buyer), json!({})).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::CONFLICT, Some("invalid_status")));
    assert_eq!(get(&db, &format!("/api/orders/{order}"), Some(&buyer)).await.json()["status"], "paid");
    assert_eq!(grants(&db, buyer_id).await, (1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_poller_confirms_payments_without_the_buyer(db: PgPool) {
    let c = catalog(&db).await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;
    let order = order_with(&db, &buyer, &c, "bakong_khqr").await;
    let (_, md5) = khqr(&db, &buyer, order).await;
    provider_reports(&md5, paid(1500, "USD", Some(BAKONG_ACCOUNT), "background"));
    routes::payments::poll_once(&state(db.clone())).await.unwrap();
    assert_eq!(get(&db, &format!("/api/orders/{order}"), Some(&buyer)).await.json()["status"], "paid");
    assert_eq!(grants(&db, buyer_id).await, (1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_approval_and_free_orders_record_how_they_were_paid(db: PgPool) {
    let c = catalog(&db).await;
    let (_, buyer) = user(&db, "+85510000001", "student").await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let order = order_both(&db, &buyer, &c).await;
    post(&db, &format!("/api/admin/orders/{order}/approve"), Some(&admin), json!({})).await;
    assert_eq!(get(&db, &format!("/api/orders/{order}"), Some(&buyer)).await.json()["paid_via"], "admin");
}
