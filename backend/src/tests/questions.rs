//! Lesson Q&A permissions.

use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn learners_ask_and_staff_answer(db: PgPool) {
    let c = catalog(&db).await;
    let (owner_id, owner) = user(&db, "+85510000002", "student").await;
    let (_, stranger) = user(&db, "+85510000001", "student").await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let (teacher_id, teacher) = user(&db, "+85510000008", "instructor").await;
    sqlx::query("UPDATE instructors SET user_id = $1").bind(teacher_id).execute(&db).await.unwrap();
    grant_both(&db, owner_id, &c).await;
    let paid = format!("/api/courses/demo-course/lessons/{}/questions", c.paid_lesson);

    // Same read access as the lesson; asking needs enrollment.
    assert_eq!(get(&db, &paid, None).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(get(&db, &paid, Some(&stranger)).await.status, StatusCode::FORBIDDEN);
    assert_eq!(post(&db, &paid, Some(&stranger), json!({ "body": "hi?" })).await.status, StatusCode::FORBIDDEN);
    assert_eq!(post(&db, &paid, Some(&owner), json!({ "body": "  " })).await.json()["error"], "invalid_body");
    let r = post(&db, &paid, Some(&owner), json!({ "body": "How does X work?" })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.text);
    assert_eq!(r.json()["can_answer"], false);
    let q = r.json()["questions"][0].clone();
    assert_eq!((q["mine"].as_bool(), q["body"].as_str()), (Some(true), Some("How does X work?")));
    let qid = q["id"].as_str().unwrap().to_owned();

    // Only staff answer: the owner can't, admins and the course's instructor can.
    let answers = format!("/api/questions/{qid}/answers");
    assert_eq!(post(&db, &answers, Some(&owner), json!({ "body": "me" })).await.status, StatusCode::FORBIDDEN);
    let queue = get(&db, "/api/admin/questions", Some(&admin)).await.json();
    assert_eq!(queue["total"], 1);
    assert_eq!(queue["items"][0]["lesson_title"], "Paid");
    let r = post(&db, &answers, Some(&teacher), json!({ "body": "Like this." })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.text);
    assert_eq!(r.json()["can_answer"], true);
    post(&db, &answers, Some(&admin), json!({ "body": "And this." })).await;
    let list = get(&db, &paid, Some(&owner)).await.json();
    assert_eq!(list["questions"][0]["answers"].as_array().unwrap().len(), 2);
    assert_eq!(get(&db, "/api/admin/questions", Some(&admin)).await.json()["total"], 0);
    assert_eq!(get(&db, "/api/admin/questions?unanswered=false", Some(&admin)).await.json()["total"], 1);
    // Staff can read Q&A without enrollment.
    assert_eq!(get(&db, &paid, Some(&teacher)).await.status, StatusCode::OK);

    // Deleting: authors and staff only.
    let aid = list["questions"][0]["answers"][0]["id"].as_str().unwrap().to_owned();
    assert_eq!(call(&db, Method::DELETE, &format!("/api/answers/{aid}"), Some(&owner), None).await.status, StatusCode::FORBIDDEN);
    assert_eq!(call(&db, Method::DELETE, &format!("/api/answers/{aid}"), Some(&teacher), None).await.status, StatusCode::NO_CONTENT);
    assert_eq!(call(&db, Method::DELETE, &format!("/api/questions/{qid}"), Some(&stranger), None).await.status, StatusCode::FORBIDDEN);
    assert_eq!(call(&db, Method::DELETE, &format!("/api/questions/{qid}"), Some(&owner), None).await.status, StatusCode::NO_CONTENT);
    assert_eq!(get(&db, &paid, Some(&owner)).await.json()["questions"], json!([]));
}
