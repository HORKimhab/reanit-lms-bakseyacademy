//! Bundles: buying one grants everything inside.

use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn buying_a_bundle_grants_its_items(db: PgPool) {
    let c = catalog(&db).await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;

    let r = post(&db, "/api/admin/bundles", Some(&admin), json!({
        "slug": "starter-pack", "title": "Starter pack", "price_cents": 1200, "published": true
    })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.text);
    let id = r.json()["id"].as_str().unwrap().to_owned();
    assert_eq!(post(&db, "/api/admin/bundles", Some(&buyer), json!({ "slug": "x", "title": "x", "price_cents": 1 })).await.status, StatusCode::FORBIDDEN);

    // An empty bundle isn't listed or sold.
    assert_eq!(get(&db, "/api/bundles", None).await.json()["total"], 0);
    assert_eq!(get(&db, "/api/bundles/starter-pack", None).await.status, StatusCode::NOT_FOUND);
    let r = post(&db, "/api/cart", Some(&buyer), json!({ "bundle_id": id })).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);

    call(&db, Method::PUT, &format!("/api/admin/bundles/{id}/courses/{}", c.course_id), Some(&admin), None).await;
    let r = call(&db, Method::PUT, &format!("/api/admin/bundles/{id}/ebooks/{}", c.ebook_id), Some(&admin), None).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    assert_eq!(r.json()["other_courses"], json!([]));
    r.assert_no_secrets(&[PAID_VIDEO, FILE_KEY, TG_FILE_ID]);

    let d = get(&db, "/api/bundles/starter-pack", None).await;
    assert_eq!(d.status, StatusCode::OK);
    let dj = d.json();
    assert_eq!((dj["item_count"].as_i64(), dj["value_cents"].as_i64(), dj["owned"].as_bool()), (Some(2), Some(1500), Some(false)));
    d.assert_no_secrets(&[PAID_VIDEO, FILE_KEY, TG_FILE_ID]);
    assert_eq!(get(&db, "/api/sitemap", None).await.json()["bundles"][0]["slug"], "starter-pack");

    // Cart → order → approval grants the course and the ebook.
    let r = post(&db, "/api/cart", Some(&buyer), json!({ "bundle_id": id })).await;
    assert_eq!(r.json()["items"][0]["kind"], "bundle");
    assert_eq!(r.json()["total_cents"], 1200);
    let r = post(&db, "/api/orders", Some(&buyer), json!({ "payment_method": "khqr" })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.text);
    let order = r.json()["id"].as_str().unwrap().to_owned();
    assert_eq!(r.json()["items"][0]["bundle_id"], id);
    let r = post(&db, "/api/cart", Some(&buyer), json!({ "bundle_id": id })).await;
    assert_eq!(r.json()["error"], "already_ordered");
    post(&db, &format!("/api/admin/orders/{order}/approve"), Some(&admin), json!({})).await;
    let owned: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM enrollments WHERE user_id = $1), (SELECT COUNT(*) FROM ebook_purchases WHERE user_id = $1)",
    )
    .bind(buyer_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(owned, (1, 1));
    assert_eq!(get(&db, "/api/bundles/starter-pack", Some(&buyer)).await.json()["owned"], true);
    assert_eq!(post(&db, "/api/cart", Some(&buyer), json!({ "bundle_id": id })).await.json()["error"], "already_owned");

    // Ordered bundles can't be deleted; owners still see an unpublished bundle.
    let r = call(&db, Method::DELETE, &format!("/api/admin/bundles/{id}"), Some(&admin), None).await;
    assert_eq!(r.json()["error"], "has_buyers");
    sqlx::query("UPDATE bundles SET published = false").execute(&db).await.unwrap();
    assert_eq!(get(&db, "/api/bundles/starter-pack", None).await.status, StatusCode::NOT_FOUND);
    assert_eq!(get(&db, "/api/bundles/starter-pack", Some(&buyer)).await.status, StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn owned_bundles_drop_out_of_the_cart_at_checkout(db: PgPool) {
    let c = catalog(&db).await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let (buyer_id, buyer) = user(&db, "+85510000001", "student").await;
    let r = post(&db, "/api/admin/bundles", Some(&admin), json!({ "slug": "solo", "title": "Solo", "price_cents": 900, "published": true })).await;
    let id = r.json()["id"].as_str().unwrap().to_owned();
    call(&db, Method::PUT, &format!("/api/admin/bundles/{id}/courses/{}", c.course_id), Some(&admin), None).await;
    post(&db, "/api/cart", Some(&buyer), json!({ "bundle_id": id })).await;
    grant_both(&db, buyer_id, &c).await;
    let r = post(&db, "/api/orders", Some(&buyer), json!({ "payment_method": "khqr" })).await;
    assert_eq!(r.json()["error"], "cart_empty");
}
