//! Paid content stays server-side until the caller owns it; owners keep access after unpublish.

use axum::http::StatusCode;
use sqlx::PgPool;

use super::*;

const ALL_SECRETS: [&str; 5] = [PAID_VIDEO, FILE_KEY, TG_FILE_ID, TG_UNIQUE_ID, BOT_TOKEN];

#[sqlx::test(migrations = "./migrations")]
async fn course_detail_exposes_only_preview_video(db: PgPool) {
    let c = catalog(&db).await;
    let (_, stranger) = user(&db, "+85510000001", "student").await;
    for token in [None, Some(stranger.as_str())] {
        let r = get(&db, "/api/courses/demo-course", token).await;
        assert_eq!(r.status, StatusCode::OK);
        r.assert_no_secrets(&ALL_SECRETS);
        assert!(r.text.contains(PREVIEW_VIDEO));
        assert_eq!(r.json()["owned"], false);
    }
    let r = get(&db, "/api/sitemap", None).await;
    assert_eq!(r.json()["courses"][0]["slug"], "demo-course");
    assert_eq!(r.json()["ebooks"][0]["slug"], "demo-ebook");
    // Even an owner's detail page never carries paid video ids: playback goes through /lessons/{id}.
    let (owner_id, owner) = user(&db, "+85510000002", "student").await;
    grant_both(&db, owner_id, &c).await;
    let r = get(&db, "/api/courses/demo-course", Some(&owner)).await;
    assert_eq!(r.json()["owned"], true);
    r.assert_no_secrets(&ALL_SECRETS);
}

#[sqlx::test(migrations = "./migrations")]
async fn paid_lessons_need_enrollment(db: PgPool) {
    let c = catalog(&db).await;
    let (_, stranger) = user(&db, "+85510000001", "student").await;
    let paid = format!("/api/courses/demo-course/lessons/{}", c.paid_lesson);
    let tg = format!("/api/courses/demo-course/lessons/{}", c.tg_lesson);
    let preview = format!("/api/courses/demo-course/lessons/{}", c.preview_lesson);

    for path in [&paid, &tg] {
        let r = get(&db, path, None).await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED);
        r.assert_no_secrets(&ALL_SECRETS);
        let r = get(&db, path, Some(&stranger)).await;
        assert_eq!(r.status, StatusCode::FORBIDDEN);
        r.assert_no_secrets(&ALL_SECRETS);
    }
    let r = get(&db, &preview, None).await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(r.json()["video_url"].as_str().unwrap().contains(PREVIEW_VIDEO));

    let r = call(&db, Method::PUT, &format!("{paid}/complete"), Some(&stranger), None).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn enrolled_users_get_signed_playback(db: PgPool) {
    let c = catalog(&db).await;
    let (owner_id, owner) = user(&db, "+85510000002", "student").await;
    grant_both(&db, owner_id, &c).await;

    let r = get(&db, &format!("/api/courses/demo-course/lessons/{}", c.paid_lesson), Some(&owner)).await;
    assert_eq!(r.status, StatusCode::OK);
    let url = r.json()["video_url"].as_str().unwrap().to_owned();
    assert!(url.contains(PAID_VIDEO) && url.contains("token="), "{url}");

    // Telegram media: only a signed /media link, never Telegram ids or the bot token.
    let r = get(&db, &format!("/api/courses/demo-course/lessons/{}", c.tg_lesson), Some(&owner)).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.json()["video_type"], "file");
    assert!(r.json()["video_url"].as_str().unwrap().starts_with("/media/"));
    r.assert_no_secrets(&ALL_SECRETS);
}

#[sqlx::test(migrations = "./migrations")]
async fn ebook_file_key_is_never_serialized(db: PgPool) {
    let c = catalog(&db).await;
    let (_, stranger) = user(&db, "+85510000001", "student").await;
    let (owner_id, owner) = user(&db, "+85510000002", "student").await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    grant_both(&db, owner_id, &c).await;

    for token in [None, Some(stranger.as_str()), Some(owner.as_str()), Some(admin.as_str())] {
        let r = get(&db, "/api/ebooks/demo-ebook", token).await;
        assert_eq!(r.status, StatusCode::OK);
        r.assert_no_secrets(&ALL_SECRETS);
    }
    for path in ["/api/ebooks?q=demo", "/api/home", "/api/me/library", "/api/recommendations"] {
        get(&db, path, Some(&owner)).await.assert_no_secrets(&ALL_SECRETS);
    }
    // Admin endpoints keep the key write-only too (`has_file`).
    let r = get(&db, &format!("/api/admin/ebooks/{}", c.ebook_id), Some(&admin)).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.json()["has_file"], true);
    r.assert_no_secrets(&ALL_SECRETS);
    get(&db, "/api/admin/ebooks", Some(&admin)).await.assert_no_secrets(&ALL_SECRETS);

    // The file endpoint: 401 / 403 for non-owners; owners pass the check (no bucket here → 503).
    assert_eq!(get(&db, "/api/ebooks/demo-ebook/file", None).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(get(&db, "/api/ebooks/demo-ebook/file", Some(&stranger)).await.status, StatusCode::FORBIDDEN);
    let r = get(&db, "/api/ebooks/demo-ebook/file", Some(&owner)).await;
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE);
    r.assert_no_secrets(&ALL_SECRETS);
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_views_hide_telegram_ids(db: PgPool) {
    let c = catalog(&db).await;
    let (_, admin) = user(&db, "+85510000009", "admin").await;
    let r = get(&db, &format!("/api/admin/courses/{}", c.course_id), Some(&admin)).await;
    assert_eq!(r.status, StatusCode::OK);
    // Admins may see Bunny video ids, but never Telegram ids or the bot token.
    assert!(r.text.contains(PAID_VIDEO));
    r.assert_no_secrets(&[FILE_KEY, TG_FILE_ID, TG_UNIQUE_ID, BOT_TOKEN]);
    let r = get(&db, "/api/admin/media", Some(&admin)).await;
    assert_eq!(r.status, StatusCode::OK);
    r.assert_no_secrets(&[TG_FILE_ID, TG_UNIQUE_ID, BOT_TOKEN]);
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_endpoints_reject_students(db: PgPool) {
    let c = catalog(&db).await;
    let (_, student) = user(&db, "+85510000001", "student").await;
    for path in [
        "/api/admin/orders".to_owned(),
        "/api/admin/media".to_owned(),
        format!("/api/admin/courses/{}", c.course_id),
        format!("/api/admin/ebooks/{}", c.ebook_id),
    ] {
        assert_eq!(get(&db, &path, None).await.status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(get(&db, &path, Some(&student)).await.status, StatusCode::FORBIDDEN, "{path}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn owners_keep_access_after_unpublish(db: PgPool) {
    let c = catalog(&db).await;
    let (_, stranger) = user(&db, "+85510000001", "student").await;
    let (owner_id, owner) = user(&db, "+85510000002", "student").await;
    grant_both(&db, owner_id, &c).await;
    sqlx::query("UPDATE courses SET published = false").execute(&db).await.unwrap();
    sqlx::query("UPDATE ebooks SET published = false").execute(&db).await.unwrap();

    let paid = format!("/api/courses/demo-course/lessons/{}", c.paid_lesson);
    let preview = format!("/api/courses/demo-course/lessons/{}", c.preview_lesson);
    for token in [None, Some(stranger.as_str())] {
        assert_eq!(get(&db, "/api/courses/demo-course", token).await.status, StatusCode::NOT_FOUND);
        assert_eq!(get(&db, "/api/ebooks/demo-ebook", token).await.status, StatusCode::NOT_FOUND);
        assert_eq!(get(&db, &preview, token).await.status, StatusCode::NOT_FOUND);
    }
    assert_eq!(get(&db, "/api/ebooks/demo-ebook/file", Some(&stranger)).await.status, StatusCode::NOT_FOUND);
    let r = get(&db, "/api/courses?q=demo", None).await;
    assert_eq!(r.json()["total"], 0);
    let r = get(&db, "/api/sitemap", None).await;
    assert_eq!(r.json(), serde_json::json!({ "courses": [], "ebooks": [], "bundles": [] }));

    assert_eq!(get(&db, "/api/courses/demo-course", Some(&owner)).await.status, StatusCode::OK);
    assert_eq!(get(&db, &paid, Some(&owner)).await.status, StatusCode::OK);
    assert_eq!(get(&db, "/api/ebooks/demo-ebook", Some(&owner)).await.status, StatusCode::OK);
    let r = call(&db, Method::PUT, &format!("{paid}/complete"), Some(&owner), None).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = call(
        &db,
        Method::PUT,
        "/api/courses/demo-course/review",
        Some(&owner),
        Some(serde_json::json!({ "rating": 5 })),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    let lib = get(&db, "/api/me/library", Some(&owner)).await.json();
    assert_eq!(lib["courses"].as_array().unwrap().len(), 1);
    assert_eq!(lib["ebooks"].as_array().unwrap().len(), 1);
}
