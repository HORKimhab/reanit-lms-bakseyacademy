use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::reviews;
use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::*,
};

pub(crate) const CARD_SELECT: &str = r#"
SELECT e.id, e.slug, e.title, e.summary, e.cover_url, e.author_name,
       e.price_cents, e.sale_price_cents,
       cat.slug AS category_slug, cat.name_km AS category_name_km, cat.name_en AS category_name_en,
       COALESCE(r.avg, 0)::float8 AS rating_avg,
       COALESCE(r.cnt, 0)::int8 AS rating_count
FROM ebooks e
LEFT JOIN categories cat ON cat.id = e.category_id
LEFT JOIN LATERAL (
    SELECT AVG(rating) AS avg, COUNT(*) AS cnt FROM reviews WHERE ebook_id = e.id
) r ON true
"#;

const FILTER: &str = r#"
WHERE e.published
  AND ($1::text IS NULL OR e.title ILIKE $1 OR e.author_name ILIKE $1 OR $3 <% e.title OR $3 <% e.author_name)
  AND ($2::text IS NULL OR cat.slug = $2)
"#;

/// Best matches first when searching (`$3` = raw query), newest first otherwise.
const ORDER: &str = "ORDER BY CASE WHEN $3::text IS NULL THEN 0
    ELSE GREATEST(word_similarity($3, e.title), word_similarity($3, e.author_name)) END DESC, e.created_at DESC";

/// Published cards matching `q` (substring, or typo-tolerant via trigram word similarity) and
/// `category`; best matches first when searching.
pub async fn query_cards(
    db: &PgPool,
    q: Option<&str>,
    pattern: Option<&str>,
    category: Option<&str>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<EbookCard>> {
    sqlx::query_as::<_, EbookCard>(&format!(
        "{CARD_SELECT} {FILTER} {ORDER} LIMIT $4 OFFSET $5"
    ))
    .bind(pattern)
    .bind(category)
    .bind(q)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn list(
    State(st): State<AppState>,
    Query(p): Query<ListParams>,
) -> AppResult<Json<Page<EbookCard>>> {
    let pattern = p.search_pattern();
    let category = p.category();

    let count_sql = format!(
        "SELECT COUNT(*) FROM ebooks e LEFT JOIN categories cat ON cat.id = e.category_id {FILTER}"
    );
    let (items, total) = tokio::try_join!(
        query_cards(&st.db, p.query(), pattern.as_deref(), category, p.per_page(), p.offset()),
        sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(pattern.as_deref())
            .bind(category)
            .bind(p.query())
            .fetch_one(&st.db),
    )?;

    Ok(Json(Page { items, total, page: p.page(), per_page: p.per_page() }))
}

pub async fn detail(
    State(st): State<AppState>,
    current: Option<CurrentUser>,
    Path(slug): Path<String>,
) -> AppResult<Json<EbookDetail>> {
    // Owners keep seeing (and reading) an ebook after it is unpublished.
    let card = sqlx::query_as::<_, EbookCard>(&format!(
        "{CARD_SELECT} WHERE e.slug = $1 AND (e.published OR EXISTS (
             SELECT 1 FROM ebook_purchases p WHERE p.ebook_id = e.id AND p.user_id = $2))"
    ))
    .bind(&slug)
    .bind(current.as_ref().map(|c| c.user.id))
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let user_id = current.as_ref().map(|c| c.user.id);
    let (extra, owned, (reviews, my_review)) = tokio::try_join!(
        sqlx::query_as::<_, EbookExtra>("SELECT description, page_count FROM ebooks WHERE id = $1")
            .bind(card.id)
            .fetch_one(&st.db),
        owns(&st.db, current.as_ref(), card.id),
        reviews::for_item(&st.db, reviews::Target::Ebook, card.id, user_id),
    )?;

    Ok(Json(EbookDetail { card, extra, owned, reviews, my_review }))
}

/// Whether the signed-in user (if any) owns this ebook.
async fn owns(db: &PgPool, current: Option<&CurrentUser>, id: Uuid) -> sqlx::Result<bool> {
    let Some(current) = current else { return Ok(false) };
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM ebook_purchases WHERE user_id = $1 AND ebook_id = $2)")
        .bind(current.user.id)
        .bind(id)
        .fetch_one(db)
        .await
}

/// Long enough for a PDF viewer to keep issuing range requests while the owner reads.
const FILE_URL_TTL: Duration = Duration::hours(1);

#[derive(Deserialize)]
pub struct FileParams {
    /// `inline` (default; for the reader) or `attachment` (download).
    disposition: Option<String>,
}

/// Presigned URL for the ebook file, for owners only (also after unpublishing).
/// 401 signed out, 403 not bought, 404 unknown / no file, 503 `storage_unavailable`.
pub async fn file(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(slug): Path<String>,
    Query(fp): Query<FileParams>,
) -> AppResult<Json<EbookFile>> {
    let attachment = match fp.disposition.as_deref() {
        None | Some("inline") => false,
        Some("attachment") => true,
        Some(_) => return Err(AppError::Validation("invalid_disposition")),
    };
    let (file_key, published, owned): (Option<String>, bool, bool) = sqlx::query_as(
        "SELECT e.file_key, e.published,
                EXISTS (SELECT 1 FROM ebook_purchases p WHERE p.ebook_id = e.id AND p.user_id = $2)
         FROM ebooks e WHERE e.slug = $1",
    )
    .bind(&slug)
    .bind(current.user.id)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    if !owned {
        return Err(if published { AppError::Forbidden } else { AppError::NotFound });
    }
    let file_key = file_key.ok_or(AppError::NotFound)?;
    let storage = st.storage.as_ref().ok_or(AppError::Unavailable("storage_unavailable"))?;

    let disposition = content_disposition(attachment, &slug, &file_key);
    let now = Utc::now();
    let url = storage.presign_get(
        &file_key,
        &[("response-content-disposition", &disposition)],
        now,
        FILE_URL_TTL.num_seconds() as u32,
    );
    Ok(Json(EbookFile { url, expires_at: now + FILE_URL_TTL }))
}

/// Download name is the (ASCII) slug plus the stored file's extension.
fn content_disposition(attachment: bool, slug: &str, file_key: &str) -> String {
    let kind = if attachment { "attachment" } else { "inline" };
    let ext = file_key
        .rsplit_once('.')
        .map(|(_, e)| e)
        .filter(|e| !e.contains('/') && e.len() <= 5 && e.bytes().all(|b| b.is_ascii_alphanumeric()));
    match ext {
        Some(ext) => format!("{kind}; filename=\"{slug}.{ext}\""),
        None => format!("{kind}; filename=\"{slug}\""),
    }
}
