//! Uploaded media stored in the Telegram channel: admin upload (and, later, library + streaming).

use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use bytes::Bytes;
use futures_util::stream;
use tokio::sync::mpsc;

use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    models::AdminMedia,
    telegram::{StoredFile, UploadKind},
};

pub(crate) const MEDIA_COLUMNS: &str = "id, provider, file_name, mime_type, size_bytes, duration_seconds, created_at";

/// `POST /api/admin/media`, multipart: a `size` field (bytes), then `file`. The file is streamed to
/// the Telegram channel as it arrives (never held in memory) and recorded in `media_files`.
/// 422 `invalid_size` / `file_too_large` / `missing_file`, 503 `telegram_unavailable` / `telegram_failed`.
pub async fn upload(State(st): State<AppState>, _: AdminUser, mut form: Multipart) -> AppResult<(StatusCode, Json<AdminMedia>)> {
    let tg = st.telegram.as_ref().ok_or(AppError::Unavailable("telegram_unavailable"))?;
    let bad_form = |_| AppError::Validation("invalid_form");
    let mut size: Option<u64> = None;
    while let Some(mut field) = form.next_field().await.map_err(bad_form)? {
        match field.name() {
            Some("size") => size = field.text().await.map_err(bad_form)?.trim().parse().ok(),
            Some("file") => {
                let size = size.filter(|s| *s > 0).ok_or(AppError::Validation("invalid_size"))?;
                if size > tg.max_upload_bytes() {
                    return Err(AppError::Validation("file_too_large"));
                }
                let file_name = safe_file_name(field.file_name().unwrap_or("file"));
                let mime = field.content_type().unwrap_or("application/octet-stream").to_owned();
                let kind = if mime.starts_with("video/") { UploadKind::Video } else { UploadKind::Document };

                // Pump multipart chunks into the request body while the upload runs.
                let (tx, rx) = mpsc::channel::<std::io::Result<Bytes>>(8);
                let body = reqwest::Body::wrap_stream(stream::unfold(rx, |mut rx| async move {
                    rx.recv().await.map(|chunk| (chunk, rx))
                }));
                // The declared `size` is the multipart length sent to Telegram, so the body must match it:
                // abort the upload as soon as it runs over, or if it ends short.
                let pump = async move {
                    let mut sent = 0u64;
                    loop {
                        let chunk = match field.chunk().await {
                            Ok(Some(chunk)) => chunk,
                            Ok(None) => break,
                            Err(e) => {
                                let _ = tx.send(Err(std::io::Error::other(e))).await;
                                return false;
                            }
                        };
                        sent += chunk.len() as u64;
                        if sent > size {
                            let _ = tx.send(Err(std::io::Error::other("body longer than declared size"))).await;
                            return false;
                        }
                        if tx.send(Ok(chunk)).await.is_err() {
                            return false;
                        }
                    }
                    if sent != size {
                        let _ = tx.send(Err(std::io::Error::other("body shorter than declared size"))).await;
                        return false;
                    }
                    true
                };
                let (complete, stored) = tokio::join!(pump, tg.upload(kind, &file_name, &mime, size, body));
                if !complete {
                    return Err(AppError::Validation("invalid_size"));
                }
                let stored = stored.map_err(|e| {
                    tracing::error!(error = %e, "telegram upload failed");
                    AppError::Unavailable("telegram_failed")
                })?;
                let media = insert_telegram(&st.db, &stored).await?;
                return Ok((StatusCode::CREATED, Json(media)));
            }
            _ => {}
        }
    }
    Err(AppError::Validation("missing_file"))
}

/// Records a Telegram file; an already-known file (same `file_unique_id`) returns the existing row.
pub(crate) async fn insert_telegram(db: &sqlx::PgPool, f: &StoredFile) -> sqlx::Result<AdminMedia> {
    sqlx::query_as::<_, AdminMedia>(&format!(
        "WITH ins AS (
             INSERT INTO media_files (provider, telegram_file_id, telegram_file_unique_id, telegram_message_id,
                                      mime_type, size_bytes, file_name, duration_seconds)
             VALUES ('telegram', $1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (telegram_file_unique_id) WHERE telegram_file_unique_id IS NOT NULL DO NOTHING
             RETURNING {MEDIA_COLUMNS}
         )
         SELECT {MEDIA_COLUMNS} FROM ins
         UNION ALL
         SELECT {MEDIA_COLUMNS} FROM media_files WHERE telegram_file_unique_id = $2 AND NOT EXISTS (SELECT 1 FROM ins)"
    ))
    .bind(&f.file_id)
    .bind(&f.file_unique_id)
    .bind(f.message_id)
    .bind(&f.mime_type)
    .bind(f.size_bytes)
    .bind(&f.file_name)
    .bind(f.duration_seconds)
    .fetch_one(db)
    .await
}

/// Keeps a display/download name: path separators and control characters dropped, ≤ 200 chars.
fn safe_file_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let cleaned: String = base.chars().filter(|c| !c.is_control() && *c != '"').take(200).collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() { "file".into() } else { cleaned.to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_are_reduced_to_a_safe_base_name() {
        assert_eq!(safe_file_name("C:\\Users\\me\\មេរៀន ១.mp4"), "មេរៀន ១.mp4");
        assert_eq!(safe_file_name("../../etc/passwd"), "passwd");
        assert_eq!(safe_file_name("a\"b\n.pdf"), "ab.pdf");
        assert_eq!(safe_file_name("  "), "file");
    }
}
