//! Saving items for later.

use axum::http::{Method, StatusCode};
use sqlx::PgPool;

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn users_save_published_items(db: PgPool) {
    let c = catalog(&db).await;
    let (_, a) = user(&db, "+85510000001", "student").await;
    let (_, b) = user(&db, "+85510000002", "student").await;
    let course = format!("/api/me/wishlist/courses/{}", c.course_id);
    let ebook = format!("/api/me/wishlist/ebooks/{}", c.ebook_id);
    for _ in 0..2 {
        assert_eq!(call(&db, Method::PUT, &course, Some(&a), None).await.status, StatusCode::NO_CONTENT);
    }
    call(&db, Method::PUT, &ebook, Some(&a), None).await;
    assert_eq!(call(&db, Method::PUT, &course, None, None).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(call(&db, Method::PUT, &format!("/api/me/wishlist/users/{}", c.course_id), Some(&a), None).await.status, StatusCode::NOT_FOUND);
    assert_eq!(call(&db, Method::PUT, &format!("/api/me/wishlist/courses/{}", c.ebook_id), Some(&a), None).await.status, StatusCode::NOT_FOUND);

    let ids = get(&db, "/api/me/wishlist/ids", Some(&a)).await.json();
    assert_eq!(ids["course_ids"][0], c.course_id.to_string());
    assert_eq!(ids["ebook_ids"][0], c.ebook_id.to_string());
    let list = get(&db, "/api/me/wishlist", Some(&a)).await;
    assert_eq!(list.json()["courses"][0]["slug"], "demo-course");
    list.assert_no_secrets(&[PAID_VIDEO, FILE_KEY]);
    assert_eq!(get(&db, "/api/me/wishlist/ids", Some(&b)).await.json()["course_ids"], serde_json::json!([]));

    // Unpublished items drop out of the list.
    sqlx::query("UPDATE ebooks SET published = false").execute(&db).await.unwrap();
    assert_eq!(get(&db, "/api/me/wishlist", Some(&a)).await.json()["ebooks"], serde_json::json!([]));
    assert_eq!(call(&db, Method::PUT, &ebook, Some(&b), None).await.status, StatusCode::NOT_FOUND);

    assert_eq!(call(&db, Method::DELETE, &course, Some(&a), None).await.status, StatusCode::NO_CONTENT);
    assert_eq!(get(&db, "/api/me/wishlist", Some(&a)).await.json()["courses"], serde_json::json!([]));
}
