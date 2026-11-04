//! Completing every lesson issues one public certificate.

use axum::http::{Method, StatusCode};
use sqlx::PgPool;

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn completing_every_lesson_issues_a_certificate(db: PgPool) {
    let c = catalog(&db).await;
    let (owner_id, owner) = user(&db, "+85510000002", "student").await;
    grant_both(&db, owner_id, &c).await;
    let complete = |id: uuid::Uuid| format!("/api/courses/demo-course/lessons/{id}/complete");
    let certificate = || async { get(&db, "/api/me/library", Some(&owner)).await.json()["courses"][0]["certificate_id"].clone() };

    for lesson in [c.preview_lesson, c.paid_lesson] {
        call(&db, Method::PUT, &complete(lesson), Some(&owner), None).await;
    }
    assert!(certificate().await.is_null());
    call(&db, Method::PUT, &complete(c.tg_lesson), Some(&owner), None).await;
    let id = certificate().await.as_str().unwrap().to_owned();

    // Idempotent, and it stays when a lesson is unmarked.
    call(&db, Method::DELETE, &complete(c.tg_lesson), Some(&owner), None).await;
    call(&db, Method::PUT, &complete(c.tg_lesson), Some(&owner), None).await;
    assert_eq!(certificate().await.as_str(), Some(id.as_str()));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM certificates").fetch_one(&db).await.unwrap();
    assert_eq!(count, 1);

    // Public verification page data.
    let r = get(&db, &format!("/api/certificates/{id}"), None).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!((r.json()["course_title"].as_str(), r.json()["full_name"].as_str()), (Some("Demo course"), Some("Test")));
    assert_eq!(get(&db, &format!("/api/certificates/{}", uuid::Uuid::max()), None).await.status, StatusCode::NOT_FOUND);
}
