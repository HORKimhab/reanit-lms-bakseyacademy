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
