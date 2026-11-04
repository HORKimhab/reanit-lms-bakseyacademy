//! Paying for an order grants its items exactly once, only through the allowed transitions.

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::*;

async fn grants(db: &PgPool, user_id: Uuid) -> (i64, i64) {
    sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM enrollments WHERE user_id = $1), \
                (SELECT COUNT(*) FROM ebook_purchases WHERE user_id = $1)",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn approval_grants_exactly_once(db: PgPool) {
    let c = catalog(&db).await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let order = order_both(&db, &buyer, &c).await;
    assert_eq!(grants(&db, buyer_id).await, (0, 0));

    // Submitting a reference doesn't grant anything yet.
    let r = post(&db, &format!("/api/orders/{order}/submit"), Some(&buyer), json!({ "payment_ref": "TX-123" })).await;
    assert_eq!(r.json()["status"], "awaiting_approval");
    assert_eq!(grants(&db, buyer_id).await, (0, 0));
    assert_eq!(
        get(&db, &format!("/api/courses/demo-course/lessons/{}", c.paid_lesson), Some(&buyer)).await.status,
        StatusCode::FORBIDDEN
    );

    // Buyers can't approve their own order.
    let approve = format!("/api/admin/orders/{order}/approve");
    assert_eq!(post(&db, &approve, Some(&buyer), json!({})).await.status, StatusCode::FORBIDDEN);

    // Concurrent and repeated approvals grant once.
    let (a, b) = tokio::join!(post(&db, &approve, Some(&admin), json!({})), post(&db, &approve, Some(&admin), json!({})));
    assert_eq!((a.status, b.status), (StatusCode::OK, StatusCode::OK));
    let r = post(&db, &approve, Some(&admin), json!({})).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.json()["status"], "paid");
    assert_eq!(grants(&db, buyer_id).await, (1, 1));
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM enrollments WHERE order_id = $1")
        .bind(order)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(rows, 1);

    assert_eq!(
        get(&db, &format!("/api/courses/demo-course/lessons/{}", c.paid_lesson), Some(&buyer)).await.status,
        StatusCode::OK
    );
    // Owned items can't be bought again.
    let r = post(&db, "/api/cart", Some(&buyer), json!({ "course_id": c.course_id })).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::CONFLICT, Some("already_owned")));
}

#[sqlx::test(migrations = "./migrations")]
async fn rejected_and_cancelled_orders_grant_nothing(db: PgPool) {
    let c = catalog(&db).await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let order = order_both(&db, &buyer, &c).await;

    // Rejecting needs a submitted order.
    let reject = format!("/api/admin/orders/{order}/reject");
    let r = post(&db, &reject, Some(&admin), json!({ "note": "no payment" })).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    post(&db, &format!("/api/orders/{order}/submit"), Some(&buyer), json!({ "payment_ref": "TX-1" })).await;
    let r = post(&db, &reject, Some(&admin), json!({ "note": "no payment" })).await;
    assert_eq!(r.json()["status"], "rejected");
    assert_eq!(grants(&db, buyer_id).await, (0, 0));

    // Resubmitting and approving grants.
    post(&db, &format!("/api/orders/{order}/submit"), Some(&buyer), json!({ "payment_ref": "TX-2" })).await;
    let r = post(&db, &format!("/api/admin/orders/{order}/approve"), Some(&admin), json!({})).await;
    assert_eq!(r.json()["status"], "paid");
    assert_eq!(grants(&db, buyer_id).await, (1, 1));

    // A cancelled order can't be approved.
    let (other_id, other) = user(&db, "+85510000002", "student").await;
    let order = order_both(&db, &other, &c).await;
    let r = post(&db, &format!("/api/orders/{order}/cancel"), Some(&other), json!({})).await;
    assert_eq!(r.json()["status"], "cancelled");
    let r = post(&db, &format!("/api/admin/orders/{order}/approve"), Some(&admin), json!({})).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(grants(&db, other_id).await, (0, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn users_only_see_their_own_orders(db: PgPool) {
    let c = catalog(&db).await;
    let (_, buyer) = user(&db, "+85510000001", "student").await;
    let (_, other) = user(&db, "+85510000002", "student").await;
    let order = order_both(&db, &buyer, &c).await;
    assert_eq!(get(&db, &format!("/api/orders/{order}"), Some(&other)).await.status, StatusCode::NOT_FOUND);
    let r = post(&db, &format!("/api/orders/{order}/submit"), Some(&other), json!({ "payment_ref": "TX-9" })).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = post(&db, &format!("/api/orders/{order}/cancel"), Some(&other), json!({})).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn buyers_are_notified_of_decisions(db: PgPool) {
    let c = catalog(&db).await;
    let (_, buyer) = user(&db, "+85510000001", "student").await;
    let (_, other) = user(&db, "+85510000002", "student").await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let order = order_both(&db, &buyer, &c).await;
    assert_eq!(get(&db, "/api/me/notifications", Some(&buyer)).await.json()["unread_count"], 0);

    post(&db, &format!("/api/orders/{order}/submit"), Some(&buyer), json!({ "payment_ref": "TX-1" })).await;
    post(&db, &format!("/api/admin/orders/{order}/reject"), Some(&admin), json!({ "note": "wrong amount" })).await;
    // A failed reject (already rejected) adds nothing.
    post(&db, &format!("/api/admin/orders/{order}/reject"), Some(&admin), json!({ "note": "again" })).await;
    let n = get(&db, "/api/me/notifications", Some(&buyer)).await.json();
    assert_eq!(n["unread_count"], 1);
    assert_eq!(n["items"][0]["kind"], "order_rejected");
    assert_eq!(n["items"][0]["note"], "wrong amount");
    assert_eq!(n["items"][0]["order_id"], order.to_string());

    // Approving twice notifies once.
    for _ in 0..2 {
        post(&db, &format!("/api/admin/orders/{order}/approve"), Some(&admin), json!({})).await;
    }
    let n = get(&db, "/api/me/notifications", Some(&buyer)).await.json();
    assert_eq!(n["unread_count"], 2);
    assert_eq!(n["items"][0]["kind"], "order_paid");
    assert_eq!(n["items"][0]["note"], serde_json::Value::Null);

    assert_eq!(get(&db, "/api/me/notifications", Some(&other)).await.json()["unread_count"], 0);
    let r = post(&db, "/api/me/notifications/read", Some(&other), json!({})).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(get(&db, "/api/me/notifications", Some(&buyer)).await.json()["unread_count"], 2);
    post(&db, "/api/me/notifications/read", Some(&buyer), json!({})).await;
    let n = get(&db, "/api/me/notifications", Some(&buyer)).await.json();
    assert_eq!((n["unread_count"].as_i64(), n["items"].as_array().map(Vec::len)), (Some(0), Some(2)));
    assert_eq!(get(&db, "/api/me/notifications", None).await.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn dashboard_counts_paid_orders(db: PgPool) {
    let c = catalog(&db).await;
    let (_, buyer) = user(&db, "+85510000001", "student").await;
    let (_, other) = user(&db, "+85510000002", "student").await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let paid = order_both(&db, &buyer, &c).await;
    let open = order_both(&db, &other, &c).await;
    post(&db, &format!("/api/orders/{open}/submit"), Some(&other), json!({ "payment_ref": "TX-2" })).await;
    post(&db, &format!("/api/admin/orders/{paid}/approve"), Some(&admin), json!({})).await;

    assert_eq!(get(&db, "/api/admin/stats", Some(&buyer)).await.status, StatusCode::FORBIDDEN);
    assert_eq!(get(&db, "/api/admin/stats?days=3", Some(&admin)).await.status, StatusCode::UNPROCESSABLE_ENTITY);
    let s = get(&db, "/api/admin/stats?days=7", Some(&admin)).await.json();
    assert_eq!(s["awaiting_approval"], 1);
    assert_eq!(s["pending_payment"], 0);
    assert_eq!(s["revenue_cents"], 1500);
    assert_eq!(s["revenue_cents_all_time"], 1500);
    assert_eq!(s["paid_orders"], 1);
    assert_eq!(s["users"], 3);
    let daily = s["daily"].as_array().unwrap();
    assert_eq!(daily.len(), 7);
    assert_eq!(daily[6]["revenue_cents"], 1500);
    assert_eq!(s["top_courses"][0]["slug"], "demo-course");
    assert_eq!(s["top_courses"][0]["sales"], 1);
    assert_eq!(s["top_ebooks"][0]["revenue_cents"], 500);
}
