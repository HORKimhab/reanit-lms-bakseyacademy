//! Admin CRUD for ebooks. `file_key` is write-only (responses carry `has_file`).

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::admin_courses::{PublishInput, set_published};
use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{AdminEbook, AdminEbookRow, ListParams, Page},
    validate,
};

const LIST_FROM: &str = r#"
FROM ebooks e
LEFT JOIN categories cat ON cat.id = e.category_id
WHERE ($1::text IS NULL OR e.title ILIKE $1 OR e.slug ILIKE $1 OR e.author_name ILIKE $1)
"#;

pub async fn list(
    State(st): State<AppState>,
    _: AdminUser,
    Query(p): Query<ListParams>,
) -> AppResult<Json<Page<AdminEbookRow>>> {
    let pattern = p.search_pattern();
    let list_sql = format!(
        "SELECT e.id, e.slug, e.title, e.author_name, e.published, e.price_cents, e.sale_price_cents,
                cat.name_km AS category_name_km, e.file_key IS NOT NULL AS has_file,
                (SELECT COUNT(*) FROM ebook_purchases p WHERE p.ebook_id = e.id) AS purchase_count,
                e.updated_at
         {LIST_FROM} ORDER BY e.updated_at DESC LIMIT $2 OFFSET $3"
    );
    let count_sql = format!("SELECT COUNT(*) {LIST_FROM}");
    let (items, total) = tokio::try_join!(
        sqlx::query_as::<_, AdminEbookRow>(&list_sql)
            .bind(pattern.as_deref())
            .bind(p.per_page())
            .bind(p.offset())
            .fetch_all(&st.db),
        sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(pattern.as_deref())
            .fetch_one(&st.db),
    )?;
    Ok(Json(Page { items, total, page: p.page(), per_page: p.per_page() }))
}

#[derive(Deserialize)]
pub struct EbookInput {
    slug: String,
    title: String,
    author_name: String,
    summary: Option<String>,
    #[serde(default)]
    description: String,
    cover_url: Option<String>,
    page_count: Option<i32>,
    category_id: Option<Uuid>,
    price_cents: i32,
    sale_price_cents: Option<i32>,
    #[serde(default)]
    published: bool,
    /// New object key; blank or absent keeps the current file.
    file_key: Option<String>,
    /// Detach the current file.
    #[serde(default)]
    remove_file: bool,
}

struct EbookFields {
    slug: String,
    title: String,
    author_name: String,
    summary: Option<String>,
    description: String,
    cover_url: Option<String>,
    page_count: Option<i32>,
    category_id: Option<Uuid>,
    price_cents: i32,
    sale_price_cents: Option<i32>,
    published: bool,
    file_key: Option<String>,
    remove_file: bool,
}

impl EbookInput {
    fn validated(self) -> AppResult<EbookFields> {
        validate::prices(self.price_cents, self.sale_price_cents)?;
        if self.page_count.is_some_and(|n| n <= 0) {
            return Err(AppError::Validation("invalid_page_count"));
        }
        Ok(EbookFields {
            slug: validate::slug(&self.slug)?,
            title: validate::required(&self.title, 200, "invalid_title")?,
            author_name: validate::required(&self.author_name, 100, "invalid_name")?,
            summary: validate::optional(self.summary.as_deref(), 500, "invalid_summary")?,
            description: validate::optional(Some(&self.description), 20_000, "invalid_description")?.unwrap_or_default(),
            cover_url: validate::optional_url(self.cover_url.as_deref())?,
            page_count: self.page_count,
            category_id: self.category_id,
            price_cents: self.price_cents,
            sale_price_cents: self.sale_price_cents,
            published: self.published,
            file_key: validate::optional(self.file_key.as_deref(), 500, "invalid_file_key")?,
            remove_file: self.remove_file,
        })
    }
}

fn map_write_error(e: sqlx::Error) -> AppError {
    match &e {
        sqlx::Error::Database(d) if d.is_unique_violation() => AppError::Conflict("slug_taken"),
        sqlx::Error::Database(d) if d.is_foreign_key_violation() => AppError::Validation("invalid_reference"),
        _ => AppError::Db(e),
    }
}

pub async fn create(
    State(st): State<AppState>,
    _: AdminUser,
    Json(input): Json<EbookInput>,
) -> AppResult<(StatusCode, Json<AdminEbook>)> {
    let f = input.validated()?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO ebooks (slug, title, author_name, summary, description, cover_url, page_count,
                             category_id, price_cents, sale_price_cents, published, file_key)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id",
    )
    .bind(f.slug)
    .bind(f.title)
    .bind(f.author_name)
    .bind(f.summary)
    .bind(f.description)
    .bind(f.cover_url)
    .bind(f.page_count)
    .bind(f.category_id)
    .bind(f.price_cents)
    .bind(f.sale_price_cents)
    .bind(f.published)
    .bind(f.file_key)
    .fetch_one(&st.db)
    .await
    .map_err(map_write_error)?;
    Ok((StatusCode::CREATED, Json(load(&st.db, id).await?)))
}

pub async fn detail(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<Json<AdminEbook>> {
    Ok(Json(load(&st.db, id).await?))
}

pub async fn update(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<EbookInput>,
) -> AppResult<Json<AdminEbook>> {
    let f = input.validated()?;
    let updated = sqlx::query(
        "UPDATE ebooks SET slug = $2, title = $3, author_name = $4, summary = $5, description = $6,
                cover_url = $7, page_count = $8, category_id = $9, price_cents = $10,
                sale_price_cents = $11, published = $12,
                file_key = CASE WHEN $14 THEN NULL ELSE COALESCE($13, file_key) END,
                updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(f.slug)
    .bind(f.title)
    .bind(f.author_name)
    .bind(f.summary)
    .bind(f.description)
    .bind(f.cover_url)
    .bind(f.page_count)
    .bind(f.category_id)
    .bind(f.price_cents)
    .bind(f.sale_price_cents)
    .bind(f.published)
    .bind(f.file_key)
    .bind(f.remove_file)
    .execute(&st.db)
    .await
    .map_err(map_write_error)?
    .rows_affected();
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(load(&st.db, id).await?))
}

pub async fn publish(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<PublishInput>,
) -> AppResult<StatusCode> {
    set_published(&st.db, "ebooks", id, input.published).await
}

/// Refused with `409 has_buyers` once anyone bought or ordered it; unpublish instead.
pub async fn delete(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let mut tx = st.db.begin().await?;
    let has_buyers: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM ebook_purchases WHERE ebook_id = $1)
             OR EXISTS (SELECT 1 FROM order_items WHERE ebook_id = $1)",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if has_buyers {
        return Err(AppError::Conflict("has_buyers"));
    }
    let deleted = sqlx::query("DELETE FROM ebooks WHERE id = $1").bind(id).execute(&mut *tx).await?.rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn load(db: &PgPool, id: Uuid) -> AppResult<AdminEbook> {
    sqlx::query_as::<_, AdminEbook>(
        "SELECT id, slug, title, author_name, summary, description, cover_url, page_count, category_id,
                price_cents, sale_price_cents, published, file_key IS NOT NULL AS has_file, updated_at
         FROM ebooks WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)
}
