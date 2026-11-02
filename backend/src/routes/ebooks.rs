use axum::{
    Json,
    extract::{Path, Query, State},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::*,
};

pub(crate) const CARD_SELECT: &str = r#"
SELECT e.id, e.slug, e.title, e.summary, e.cover_url, e.author_name,
       e.price_cents, e.sale_price_cents,
       cat.slug AS category_slug, cat.name_km AS category_name_km,
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
  AND ($1::text IS NULL OR e.title ILIKE $1 OR e.author_name ILIKE $1)
  AND ($2::text IS NULL OR cat.slug = $2)
"#;

pub async fn query_cards(
    db: &PgPool,
    pattern: Option<&str>,
    category: Option<&str>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<EbookCard>> {
    sqlx::query_as::<_, EbookCard>(&format!(
        "{CARD_SELECT} {FILTER} ORDER BY e.created_at DESC LIMIT $3 OFFSET $4"
    ))
    .bind(pattern)
    .bind(category)
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
        query_cards(&st.db, pattern.as_deref(), category, p.per_page(), p.offset()),
        sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(pattern.as_deref())
            .bind(category)
            .fetch_one(&st.db),
    )?;

    Ok(Json(Page { items, total, page: p.page(), per_page: p.per_page() }))
}

pub async fn detail(
    State(st): State<AppState>,
    current: Option<CurrentUser>,
    Path(slug): Path<String>,
) -> AppResult<Json<EbookDetail>> {
    let card = sqlx::query_as::<_, EbookCard>(&format!(
        "{CARD_SELECT} WHERE e.published AND e.slug = $1"
    ))
    .bind(&slug)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let (extra, owned) = tokio::try_join!(
        sqlx::query_as::<_, EbookExtra>("SELECT description, page_count FROM ebooks WHERE id = $1")
            .bind(card.id)
            .fetch_one(&st.db),
        owns(&st.db, current.as_ref(), card.id),
    )?;

    Ok(Json(EbookDetail { card, extra, owned }))
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
