//! Media stored in Telegram: admin upload, channel-post import (webhook), the media library, and
//! streaming through signed `/media/{token}` links.

use axum::{
    Json,
    body::{Body, Bytes as BodyBytes},
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;
use bytes::Bytes;
use futures_util::stream;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    media_token::{self, MediaGrant},
    models::{AdminMedia, ListParams, Page},
    telegram::{OpenError, StoredFile, Telegram, UploadKind, channel_post_file},
};

pub(crate) const MEDIA_COLUMNS: &str = "id, provider, file_name, mime_type, size_bytes, duration_seconds, created_at";

/// `POST /api/admin/media`, multipart: a `size` field (bytes), then `file`. The file is streamed to
/// the Telegram channel as it arrives (never held in memory) and recorded in `media_files`.
/// 422 `invalid_size` / `file_too_large` / `missing_file`, 503 `telegram_unavailable` / `telegram_failed`.
pub async fn upload(State(st): State<AppState>, _: AdminUser, form: Multipart) -> AppResult<(StatusCode, Json<AdminMedia>)> {
    let tg = st.telegram.as_ref().ok_or(AppError::Unavailable("telegram_unavailable"))?;
    let check = |_: &str, size: u64| {
        if size > tg.max_upload_bytes() { Err(AppError::Validation("file_too_large")) } else { Ok(()) }
    };
    let stored = upload_form_to_telegram(tg, form, check, tg.channel_id(), None).await?;
    let media = with_preview(&st, insert_telegram(&st.db, &stored, Purpose::Content).await?);
    Ok((StatusCode::CREATED, Json(media)))
}

/// Reads a multipart form of `size` (bytes) then `file`, checks `(mime, size)` with `check`, and
/// streams the file to `chat_id` as it arrives. The body must be exactly `size` bytes (it is the
/// length announced to Telegram): longer or shorter aborts with 422 `invalid_size`.
pub(crate) async fn upload_form_to_telegram(
    tg: &Telegram,
    mut form: Multipart,
    check: impl Fn(&str, u64) -> AppResult<()>,
    chat_id: &str,
    caption: Option<&str>,
) -> AppResult<StoredFile> {
    let bad_form = |_| AppError::Validation("invalid_form");
    let mut size: Option<u64> = None;
    while let Some(mut field) = form.next_field().await.map_err(bad_form)? {
        match field.name() {
            Some("size") => size = field.text().await.map_err(bad_form)?.trim().parse().ok(),
            Some("file") => {
                let size = size.filter(|s| *s > 0).ok_or(AppError::Validation("invalid_size"))?;
                let mime = field.content_type().unwrap_or("application/octet-stream").to_owned();
                check(&mime, size)?;
                let file_name = safe_file_name(field.file_name().unwrap_or("file"));
                let kind = if mime.starts_with("video/") { UploadKind::Video } else { UploadKind::Document };

                // Pump multipart chunks into the request body while the upload runs.
                let (tx, rx) = mpsc::channel::<std::io::Result<Bytes>>(8);
                let body = reqwest::Body::wrap_stream(stream::unfold(rx, |mut rx| async move {
                    rx.recv().await.map(|chunk| (chunk, rx))
                }));
                let pump = async move {
                    let mut sent = 0u64;
                    loop {
                        let chunk = match field.chunk().await {
                            Ok(Some(chunk)) => chunk,
                            Ok(None) => break,
                            Err(e) => {
                                let _ = tx.send(Err(std::io::Error::other(e))).await;
                                return Pump::ClientError;
                            }
                        };
                        sent += chunk.len() as u64;
                        if sent > size {
                            let _ = tx.send(Err(std::io::Error::other("body longer than declared size"))).await;
                            return Pump::SizeMismatch;
                        }
                        if tx.send(Ok(chunk)).await.is_err() {
                            return Pump::UploadGone;
                        }
                    }
                    if sent != size {
                        let _ = tx.send(Err(std::io::Error::other("body shorter than declared size"))).await;
                        return Pump::SizeMismatch;
                    }
                    Pump::Done
                };
                let upload = tg.upload_to(chat_id, caption, kind, &file_name, &mime, size, body);
                let (outcome, stored) = tokio::join!(pump, upload);
                return match (outcome, stored) {
                    (Pump::SizeMismatch, _) => Err(AppError::Validation("invalid_size")),
                    (Pump::ClientError, _) => Err(AppError::Validation("invalid_form")),
                    // Telegram failed (or hung up, which stops the pump): their error, not the client's.
                    (_, Err(e)) => {
                        tracing::error!(error = %e, "telegram upload failed");
                        Err(AppError::Unavailable("telegram_failed"))
                    }
                    (Pump::Done | Pump::UploadGone, Ok(stored)) => Ok(stored),
                };
            }
            _ => {}
        }
    }
    Err(AppError::Validation("missing_file"))
}

/// How feeding the client's file into the Telegram upload ended.
enum Pump {
    Done,
    /// The body was longer or shorter than the declared `size`.
    SizeMismatch,
    /// The client's multipart stream broke.
    ClientError,
    /// The upload stopped reading (Telegram failed or hung up).
    UploadGone,
}

/// Records a Telegram file; an already-known file (same `file_unique_id`) returns the existing row.
/// What a media row is for: course content (the media library) or a buyer's payment screenshot.
#[derive(Clone, Copy)]
pub(crate) enum Purpose {
    Content,
    PaymentProof,
}

pub(crate) async fn insert_telegram(db: &sqlx::PgPool, f: &StoredFile, purpose: Purpose) -> sqlx::Result<AdminMedia> {
    sqlx::query_as::<_, AdminMedia>(&format!(
        "WITH ins AS (
             INSERT INTO media_files (provider, telegram_file_id, telegram_file_unique_id, telegram_message_id,
                                      mime_type, size_bytes, file_name, duration_seconds, purpose)
             VALUES ('telegram', $1, $2, $3, $4, $5, $6, $7, $8)
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
    .bind(match purpose {
        Purpose::Content => "content",
        Purpose::PaymentProof => "payment_proof",
    })
    .fetch_one(db)
    .await
}

#[derive(Deserialize)]
pub struct MediaListParams {
    /// `video` or `document` (anything not a video).
    kind: Option<String>,
}

const LIST_FILTER: &str = "WHERE purpose = 'content'
    AND ($1::text IS NULL OR file_name ILIKE $1)
    AND ($2::text IS NULL OR (mime_type LIKE 'video/%') = ($2 = 'video'))";

/// `GET /api/admin/media?q=&kind=video|document&page=&per_page=` (≤ 200): the media library of
/// course content, newest first; `q` searches file names. Payment screenshots are never listed.
pub async fn list(
    State(st): State<AppState>,
    _: AdminUser,
    Query(p): Query<ListParams>,
    Query(mp): Query<MediaListParams>,
) -> AppResult<Json<Page<AdminMedia>>> {
    let kind = match mp.kind.as_deref() {
        None | Some("") => None,
        Some(k @ ("video" | "document")) => Some(k),
        Some(_) => return Err(AppError::Validation("invalid_kind")),
    };
    let pattern = p.search_pattern();
    let per_page = p.per_page.unwrap_or(30).clamp(1, 200);
    let offset = (p.page() - 1) * per_page;
    let list_sql = format!("SELECT {MEDIA_COLUMNS} FROM media_files {LIST_FILTER} ORDER BY created_at DESC LIMIT $3 OFFSET $4");
    let count_sql = format!("SELECT COUNT(*) FROM media_files {LIST_FILTER}");
    let (items, total) = tokio::try_join!(
        sqlx::query_as::<_, AdminMedia>(&list_sql)
            .bind(pattern.as_deref())
            .bind(kind)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&st.db),
        sqlx::query_scalar::<_, i64>(&count_sql).bind(pattern.as_deref()).bind(kind).fetch_one(&st.db),
    )?;
    let items = items.into_iter().map(|m| with_preview(&st, m)).collect();
    Ok(Json(Page { items, total, page: p.page(), per_page }))
}

fn with_preview(st: &AppState, mut media: AdminMedia) -> AdminMedia {
    if st.telegram.is_some() && media.provider == "telegram" {
        media.preview_url = Some(signed_url(st, media.id, Duration::hours(1), false).0);
    }
    media
}

/// `POST /api/telegram/webhook`: Telegram delivers channel posts here (`setWebhook` with
/// `secret_token`). Videos/documents posted in the storage channel become `media_files` rows.
/// Anything else is acknowledged with 200 so Telegram doesn't retry it.
pub async fn webhook(State(st): State<AppState>, headers: HeaderMap, body: BodyBytes) -> AppResult<StatusCode> {
    let tg = st.telegram.as_ref().ok_or(AppError::NotFound)?;
    let secret = headers.get("x-telegram-bot-api-secret-token").map(|v| v.as_bytes());
    if !tg.webhook_authorized(secret) {
        return Err(AppError::Unauthorized("unauthorized"));
    }
    if let Some(file) = channel_post_file(&body, tg.channel_id()) {
        let media = insert_telegram(&st.db, &file, Purpose::Content).await?;
        tracing::info!(media_id = %media.id, file_name = ?media.file_name, "imported channel post");
    }
    Ok(StatusCode::OK)
}

/// A site-relative link (`/media/{token}`, served by the SvelteKit proxy) to stream `media_id`
/// for `ttl`, and when it stops working.
pub(crate) fn signed_url(st: &AppState, media_id: Uuid, ttl: Duration, attachment: bool) -> (String, DateTime<Utc>) {
    let expires = Utc::now() + ttl;
    let token = media_token::sign(&st.media_key, &MediaGrant { media_id, expires: expires.timestamp(), attachment });
    (format!("/media/{token}"), expires)
}

/// `GET /api/media/{token}`: streams a Telegram-stored file the token grants, honoring `Range`
/// (video seeking, PDF viewers). Invalid/expired tokens and unknown media are 404 alike.
pub async fn stream(State(st): State<AppState>, Path(token): Path<String>, headers: HeaderMap) -> AppResult<Response> {
    let grant = media_token::verify(&st.media_key, &token, Utc::now().timestamp()).ok_or(AppError::NotFound)?;
    let tg = st.telegram.as_ref().ok_or(AppError::Unavailable("telegram_unavailable"))?;
    let (file_id, mime, file_name, size): (Option<String>, Option<String>, Option<String>, Option<i64>) =
        sqlx::query_as(
            "SELECT telegram_file_id, mime_type, file_name, size_bytes FROM media_files
             WHERE id = $1 AND provider = 'telegram'",
        )
        .bind(grant.media_id)
        .fetch_optional(&st.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let file_id = file_id.ok_or(AppError::NotFound)?;
    let range = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    let telegram_failed = |e: anyhow::Error| {
        tracing::error!(error = %e, media_id = %grant.media_id, "telegram stream failed");
        AppError::Unavailable("telegram_failed")
    };
    // A cached download path can go stale; retry once with a fresh one.
    let mut opened = None;
    for fresh in [false, true] {
        let path = tg.file_path_cached(&file_id, fresh).await.map_err(telegram_failed)?;
        match tg.open(&path, range).await {
            Ok(body) => {
                opened = Some(body);
                break;
            }
            Err(OpenError::RangeNotSatisfiable) => {
                let mut res = StatusCode::RANGE_NOT_SATISFIABLE.into_response();
                if let Some(Ok(v)) = size.map(|s| HeaderValue::from_str(&format!("bytes */{s}"))) {
                    res.headers_mut().insert(header::CONTENT_RANGE, v);
                }
                return Ok(res);
            }
            Err(OpenError::Other(e)) if fresh => return Err(telegram_failed(e)),
            Err(OpenError::Other(_)) => continue,
        }
    }
    let body = opened.ok_or(AppError::NotFound)?;

    let mut res = Response::new(Body::from_stream(body.stream));
    *res.status_mut() = body.status;
    let h = res.headers_mut();
    let safe = mime.as_deref().is_some_and(inline_safe);
    let content_type = mime.as_deref().filter(|_| safe).and_then(|m| HeaderValue::from_str(m).ok());
    h.insert(header::CONTENT_TYPE, content_type.unwrap_or(HeaderValue::from_static("application/octet-stream")));
    // Unsafe types are forced to download; if a browser renders one anyway, it gets no script or
    // origin powers. (Not on inline media: a sandboxed document blocks Chrome's built-in PDF viewer.)
    if !safe {
        h.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("sandbox; default-src 'none'"));
    }
    h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=3600"));
    h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    if let Some(len) = body.content_length {
        h.insert(header::CONTENT_LENGTH, len.into());
    }
    if let Some(Ok(v)) = body.content_range.map(|r| HeaderValue::from_str(&r)) {
        h.insert(header::CONTENT_RANGE, v);
    }
    let attachment = grant.attachment || !safe;
    if let Ok(v) = HeaderValue::from_str(&content_disposition(attachment, file_name.as_deref().unwrap_or("file"))) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    Ok(res)
}

/// Types safe to render inline on the site's origin. Anything else (HTML, SVG, scripts…) is
/// always served as a download so an uploaded file can't run in the site's context.
fn inline_safe(mime: &str) -> bool {
    let plain = mime.bytes().all(|b| b.is_ascii_alphanumeric() || b"/+-.".contains(&b));
    plain
        && (mime.starts_with("video/")
            || mime.starts_with("audio/")
            || matches!(mime, "application/pdf" | "image/png" | "image/jpeg" | "image/webp" | "image/gif"))
}

/// `inline` / `attachment` with an ASCII fallback name and the real (e.g. Khmer) name per RFC 6266.
pub(crate) fn content_disposition(attachment: bool, file_name: &str) -> String {
    let kind = if attachment { "attachment" } else { "inline" };
    let ascii: String = file_name
        .chars()
        .map(|c| if c.is_ascii_graphic() && c != '"' && c != '\\' || c == ' ' { c } else { '_' })
        .collect();
    let encoded: String = file_name
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect();
    format!("{kind}; filename=\"{ascii}\"; filename*=UTF-8''{encoded}")
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
    fn only_passive_types_render_inline() {
        for ok in ["video/mp4", "video/webm", "audio/mpeg", "application/pdf", "image/png", "image/jpeg", "image/webp", "image/gif"] {
            assert!(inline_safe(ok), "{ok}");
        }
        for bad in ["text/html", "image/svg+xml", "application/xhtml+xml", "text/javascript", "application/octet-stream", "", "video/mp4; x=<"] {
            assert!(!inline_safe(bad), "{bad}");
        }
    }

    #[test]
    fn disposition_keeps_unicode_names_via_filename_star() {
        assert_eq!(
            content_disposition(true, "មេរៀន 1.pdf"),
            "attachment; filename=\"_____ 1.pdf\"; filename*=UTF-8''%E1%9E%98%E1%9F%81%E1%9E%9A%E1%9F%80%E1%9E%93%201.pdf"
        );
        assert_eq!(content_disposition(false, "a\"b.mp4"), "inline; filename=\"a_b.mp4\"; filename*=UTF-8''a%22b.mp4");
    }

    #[test]
    fn file_names_are_reduced_to_a_safe_base_name() {
        assert_eq!(safe_file_name("C:\\Users\\me\\មេរៀន ១.mp4"), "មេរៀន ១.mp4");
        assert_eq!(safe_file_name("../../etc/passwd"), "passwd");
        assert_eq!(safe_file_name("a\"b\n.pdf"), "ab.pdf");
        assert_eq!(safe_file_name("  "), "file");
    }
}
