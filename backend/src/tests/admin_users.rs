//! Admin user management: roles and manual grants.

use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn admins_grant_revoke_and_promote(db: PgPool) {
    let c = catalog(&db).await;
    let (admin_id, admin) = user(&db, "+85510000009", "admin").await;
    let (student_id, student) = user(&db, "+85510000001", "student").await;
    let base = format!("/api/admin/users/{student_id}");
    let lesson = format!("/api/courses/demo-course/lessons/{}", c.paid_lesson);

    assert_eq!(get(&db, "/api/admin/users", Some(&student)).await.status, StatusCode::FORBIDDEN);
    let list = get(&db, "/api/admin/users?q=10000001", Some(&admin)).await.json();
    assert_eq!((list["total"].as_i64(), list["items"][0]["id"].as_str()), (Some(1), Some(student_id.to_string().as_str())));
    assert_eq!(get(&db, "/api/admin/users?role=admin", Some(&admin)).await.json()["total"], 1);

    // Grant: access without an order, item leaves the cart, idempotent.
    post(&db, "/api/cart", Some(&student), json!({ "course_id": c.course_id })).await;
    let grant = format!("{base}/courses/{}", c.course_id);
    for _ in 0..2 {
        assert_eq!(call(&db, Method::PUT, &grant, Some(&admin), None).await.status, StatusCode::NO_CONTENT);
    }
    assert_eq!(get(&db, &lesson, Some(&student)).await.status, StatusCode::OK);
    assert_eq!(get(&db, "/api/cart", Some(&student)).await.json()["items"], json!([]));
    call(&db, Method::PUT, &format!("{base}/ebooks/{}", c.ebook_id), Some(&admin), None).await;
    let d = get(&db, &base, Some(&admin)).await;
    let d_json = d.json();
    assert_eq!(d_json["courses"][0]["order_id"], serde_json::Value::Null);
    assert_eq!((d_json["course_count"].as_i64(), d_json["ebook_count"].as_i64()), (Some(1), Some(1)));
    assert_eq!(d_json["grantable_courses"], json!([]));
    d.assert_no_secrets(&[PAID_VIDEO, FILE_KEY, TG_FILE_ID]);
    assert_eq!(call(&db, Method::PUT, &format!("{base}/courses/{}", c.ebook_id), Some(&admin), None).await.status, StatusCode::NOT_FOUND);
    assert_eq!(call(&db, Method::PUT, &format!("{base}/users/{}", c.course_id), Some(&admin), None).await.status, StatusCode::NOT_FOUND);

    // Revoke.
    assert_eq!(call(&db, Method::DELETE, &grant, Some(&admin), None).await.status, StatusCode::NO_CONTENT);
    assert_eq!(get(&db, &lesson, Some(&student)).await.status, StatusCode::FORBIDDEN);

    // Roles: promote others, never yourself.
    let role = |id: uuid::Uuid| format!("/api/admin/users/{id}/role");
    let r = call(&db, Method::PUT, &role(student_id), Some(&admin), Some(json!({ "role": "root" }))).await;
    assert_eq!(r.json()["error"], "invalid_role");
    let r = call(&db, Method::PUT, &role(admin_id), Some(&admin), Some(json!({ "role": "student" }))).await;
    assert_eq!((r.status, r.json()["error"].as_str()), (StatusCode::CONFLICT, Some("own_role")));
    let r = call(&db, Method::PUT, &role(student_id), Some(&admin), Some(json!({ "role": "admin" }))).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(get(&db, "/api/admin/users", Some(&student)).await.status, StatusCode::OK);
}
