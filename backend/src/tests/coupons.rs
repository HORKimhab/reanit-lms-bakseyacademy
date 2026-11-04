//! Discount codes: preview in the cart, applied once at checkout, limits enforced.

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::*;

async fn fill_cart(db: &PgPool, token: &str, c: &Catalog) {
    post(db, "/api/cart", Some(token), json!({ "course_id": c.course_id })).await;
    post(db, "/api/cart", Some(token), json!({ "ebook_id": c.ebook_id })).await;
}

fn code(r: &Resp) -> String {
    r.json()["error"].as_str().unwrap_or_default().to_owned()
}

#[sqlx::test(migrations = "./migrations")]
async fn coupons_discount_the_order(db: PgPool) {
    let c = catalog(&db).await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let (_, buyer) = user(&db, "+85510000001", "student").await;
    let r = post(&db, "/api/admin/coupons", Some(&admin), json!({ "code": "save10", "percent_off": 10 })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.text);
    assert_eq!(r.json()["code"], "SAVE10");
    let r = post(&db, "/api/admin/coupons", Some(&admin), json!({ "code": "SAVE10", "amount_off_cents": 100 })).await;
    assert_eq!((r.status, code(&r).as_str()), (StatusCode::CONFLICT, "code_taken"));
    let r = post(&db, "/api/admin/coupons", Some(&admin), json!({ "code": "BOTH", "percent_off": 5, "amount_off_cents": 100 })).await;
    assert_eq!(code(&r), "invalid_discount");
    assert_eq!(post(&db, "/api/admin/coupons", Some(&buyer), json!({ "code": "X1X", "percent_off": 5 })).await.status, StatusCode::FORBIDDEN);

    fill_cart(&db, &buyer, &c).await;
    let cart = get(&db, "/api/cart?coupon=save10", Some(&buyer)).await.json();
    assert_eq!((cart["subtotal_cents"].as_i64(), cart["discount_cents"].as_i64(), cart["total_cents"].as_i64()), (Some(1500), Some(150), Some(1350)));
    assert_eq!(cart["coupon"]["code"], "SAVE10");
    assert_eq!(code(&get(&db, "/api/cart?coupon=NOPE", Some(&buyer)).await), "invalid_coupon");

    let r = post(&db, "/api/orders", Some(&buyer), json!({ "payment_method": "khqr", "coupon_code": "Save10" })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.text);
    let o = r.json();
    assert_eq!((o["total_cents"].as_i64(), o["discount_cents"].as_i64()), (Some(1350), Some(150)));
    assert_eq!(o["coupon_code"], "SAVE10");
    let coupons = get(&db, "/api/admin/coupons", Some(&admin)).await.json();
    assert_eq!(coupons[0]["uses"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn coupon_limits_are_enforced(db: PgPool) {
    let c = catalog(&db).await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let (_, a) = user(&db, "+85510000001", "student").await;
    let (_, b) = user(&db, "+85510000002", "student").await;
    post(&db, "/api/admin/coupons", Some(&admin), json!({ "code": "ONCE", "amount_off_cents": 5000, "max_uses": 1 })).await;
    post(&db, "/api/admin/coupons", Some(&admin), json!({ "code": "OLD", "percent_off": 50, "expires_at": "2020-01-01T00:00:00Z" })).await;
    let off = post(&db, "/api/admin/coupons", Some(&admin), json!({ "code": "OFF", "percent_off": 50, "active": false })).await;
    assert_eq!(off.json()["active"], false);

    fill_cart(&db, &a, &c).await;
    fill_cart(&db, &b, &c).await;
    for (coupon, err) in [("OLD", "coupon_expired"), ("OFF", "invalid_coupon")] {
        let r = post(&db, "/api/orders", Some(&a), json!({ "payment_method": "khqr", "coupon_code": coupon })).await;
        assert_eq!(code(&r), err);
    }

    // The fixed amount exceeds the subtotal: the order becomes free and is granted at once.
    let r = post(&db, "/api/orders", Some(&a), json!({ "coupon_code": "ONCE" })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.text);
    assert_eq!((r.json()["status"].as_str(), r.json()["total_cents"].as_i64()), (Some("paid"), Some(0)));
    assert_eq!(get(&db, &format!("/api/courses/demo-course/lessons/{}", c.paid_lesson), Some(&a)).await.status, StatusCode::OK);

    let r = post(&db, "/api/orders", Some(&b), json!({ "payment_method": "khqr", "coupon_code": "ONCE" })).await;
    assert_eq!(code(&r), "coupon_used_up");
    assert_eq!(code(&get(&db, "/api/cart?coupon=ONCE", Some(&b)).await), "coupon_used_up");
    // A failed coupon leaves the cart intact.
    assert_eq!(get(&db, "/api/cart", Some(&b)).await.json()["items"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_checkouts_share_one_last_use(db: PgPool) {
    let c = catalog(&db).await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let (_, a) = user(&db, "+85510000001", "student").await;
    let (_, b) = user(&db, "+85510000002", "student").await;
    post(&db, "/api/admin/coupons", Some(&admin), json!({ "code": "LAST", "percent_off": 20, "max_uses": 1 })).await;
    fill_cart(&db, &a, &c).await;
    fill_cart(&db, &b, &c).await;
    let body = json!({ "payment_method": "khqr", "coupon_code": "LAST" });
    let (ra, rb) = tokio::join!(post(&db, "/api/orders", Some(&a), body.clone()), post(&db, "/api/orders", Some(&b), body.clone()));
    let mut statuses = [ra.status, rb.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::CREATED, StatusCode::UNPROCESSABLE_ENTITY], "{} / {}", ra.text, rb.text);

    // Cancelling frees the use again.
    let winner = if ra.status == StatusCode::CREATED { (&ra, &a) } else { (&rb, &b) };
    let id = winner.0.json()["id"].as_str().unwrap().to_owned();
    post(&db, &format!("/api/orders/{id}/cancel"), Some(winner.1), json!({})).await;
    let loser = if winner.1 == &a { &b } else { &a };
    assert_eq!(get(&db, "/api/cart?coupon=LAST", Some(loser)).await.status, StatusCode::OK);
}
