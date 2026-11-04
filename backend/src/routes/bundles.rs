//! Bundles: several courses / ebooks sold at one price. Public catalog and admin CRUD.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{courses, ebooks, orders::bundle_item_not_owned};
use crate::{
    AppState,
    auth::{AdminUser, CurrentUser},
    error::{AppError, AppResult},
    models::{
        AdminBundle, AdminBundleRow, BundleCard, BundleDetail, BundleExtra, CourseCard, EbookCard, ItemOption,
        ListParams, Page,
    },
    validate,
};

/// Card columns; `value_cents` is what the items would cost one by one (sale prices).
pub(crate) const CARD_SELECT: &str = r#"
SELECT b.id, b.slug, b.title, b.summary, b.cover_url, b.price_cents, b.sale_price_cents,
       (SELECT COUNT(*) FROM bundle_items bi WHERE bi.bundle_id = b.id) AS item_count,
       (SELECT COALESCE(SUM(COALESCE(c.sale_price_cents, c.price_cents, e.sale_price_cents, e.price_cents)), 0)::int8
        FROM bundle_items bi LEFT JOIN courses c ON c.id = bi.course_id LEFT JOIN ebooks e ON e.id = bi.ebook_id
        WHERE bi.bundle_id = b.id) AS value_cents
FROM bundles b
"#;

/// Published bundles with at least one item.
const PUBLIC: &str = "b.published AND EXISTS (SELECT 1 FROM bundle_items bi WHERE bi.bundle_id = b.id)";

pub async fn list(State(st): State<AppState>, Query(p): Query<ListParams>) -> AppResult<Json<Page<BundleCard>>> {
    let list_sql = format!("{CARD_SELECT} WHERE {PUBLIC} ORDER BY b.created_at DESC LIMIT $1 OFFSET $2");
    let count_sql = format!("SELECT COUNT(*) FROM bundles b WHERE {PUBLIC}");
    let (items, total) = tokio::try_join!(
        sqlx::query_as::<_, BundleCard>(&list_sql).bind(p.per_page()).bind(p.offset()).fetch_all(&st.db),
        sqlx::query_scalar::<_, i64>(&count_sql).fetch_one(&st.db),
    )?;
    Ok(Json(Page { items, total, page: p.page(), per_page: p.per_page() }))
}

/// Contents of a bundle in admin order (cards carry no paid content).
async fn contents(db: &PgPool, bundle_id: Uuid) -> sqlx::Result<(Vec<CourseCard>, Vec<EbookCard>)> {
    let course_sql = format!(
        "{} JOIN bundle_items bi ON bi.course_id = c.id AND bi.bundle_id = $1 ORDER BY bi.position, c.title",
        courses::CARD_SELECT
    );
    let ebook_sql = format!(
        "{} JOIN bundle_items bi ON bi.ebook_id = e.id AND bi.bundle_id = $1 ORDER BY bi.position, e.title",
        ebooks::CARD_SELECT
    );
    tokio::try_join!(
        sqlx::query_as::<_, CourseCard>(&course_sql).bind(bundle_id).fetch_all(db),
        sqlx::query_as::<_, EbookCard>(&ebook_sql).bind(bundle_id).fetch_all(db),
    )
}

/// A published bundle (or one whose contents the caller owns entirely) with what's inside.
pub async fn detail(
    State(st): State<AppState>,
    current: Option<CurrentUser>,
    Path(slug): Path<String>,
) -> AppResult<Json<BundleDetail>> {
    let user_id = current.as_ref().map(|c| c.user.id);
    let owned_sql = format!("($2::uuid IS NOT NULL AND NOT EXISTS ({}))", bundle_item_not_owned("b.id", "$2"));
    let id: Uuid = sqlx::query_scalar(&format!(
        "SELECT b.id FROM bundles b WHERE b.slug = $1 AND ({PUBLIC} OR {owned_sql})"
    ))
    .bind(&slug)
    .bind(user_id)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    let card_sql = format!("{CARD_SELECT} WHERE b.id = $1");
    let is_owned_sql = format!("SELECT {owned_sql} FROM bundles b WHERE b.id = $1");
    let (card, extra, owned, (courses, ebooks)) = tokio::try_join!(
        sqlx::query_as::<_, BundleCard>(&card_sql).bind(id).fetch_one(&st.db),
        sqlx::query_as::<_, BundleExtra>("SELECT description FROM bundles WHERE id = $1").bind(id).fetch_one(&st.db),
        sqlx::query_scalar::<_, bool>(&is_owned_sql)
            .bind(id)
            .bind(user_id)
            .fetch_one(&st.db),
        contents(&st.db, id),
    )?;
    Ok(Json(BundleDetail { card, extra, owned, courses, ebooks }))
}

// ---- Admin ---------------------------------------------------------------------------------

const ADMIN_FIELDS: &str = "b.id, b.slug, b.title, b.summary, b.description, b.cover_url, b.price_cents, \
     b.sale_price_cents, b.published, b.updated_at";

pub async fn admin_list(
    State(st): State<AppState>,
    _: AdminUser,
    Query(p): Query<ListParams>,
) -> AppResult<Json<Page<AdminBundleRow>>> {
    let filter = "FROM bundles b WHERE ($1::text IS NULL OR b.title ILIKE $1 OR b.slug ILIKE $1)";
    let pattern = p.search_pattern();
    let list_sql = format!(
        "SELECT b.id, b.slug, b.title, b.published, b.price_cents, b.sale_price_cents, b.updated_at,
                (SELECT COUNT(*) FROM bundle_items bi WHERE bi.bundle_id = b.id) AS item_count,
                (SELECT COUNT(*) FROM order_items oi JOIN orders o ON o.id = oi.order_id
                 WHERE oi.bundle_id = b.id AND o.status = 'paid') AS sales
         {filter} ORDER BY b.updated_at DESC LIMIT $2 OFFSET $3"
    );
    let count_sql = format!("SELECT COUNT(*) {filter}");
    let (items, total) = tokio::try_join!(
        sqlx::query_as::<_, AdminBundleRow>(&list_sql)
        .bind(pattern.as_deref())
        .bind(p.per_page())
        .bind(p.offset())
        .fetch_all(&st.db),
        sqlx::query_scalar::<_, i64>(&count_sql).bind(pattern.as_deref()).fetch_one(&st.db),
    )?;
    Ok(Json(Page { items, total, page: p.page(), per_page: p.per_page() }))
}

async fn admin_load(db: &PgPool, id: Uuid) -> AppResult<AdminBundle> {
    let fields = sqlx::query_as(&format!("SELECT {ADMIN_FIELDS} FROM bundles b WHERE b.id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::NotFound)?;
    let ((courses, ebooks), other_courses, other_ebooks) = tokio::try_join!(
        contents(db, id),
        sqlx::query_as::<_, ItemOption>(
            "SELECT id, title, published FROM courses c
             WHERE NOT EXISTS (SELECT 1 FROM bundle_items bi WHERE bi.bundle_id = $1 AND bi.course_id = c.id)
             ORDER BY title",
        )
        .bind(id)
        .fetch_all(db),
        sqlx::query_as::<_, ItemOption>(
            "SELECT id, title, published FROM ebooks e
             WHERE NOT EXISTS (SELECT 1 FROM bundle_items bi WHERE bi.bundle_id = $1 AND bi.ebook_id = e.id)
             ORDER BY title",
        )
        .bind(id)
        .fetch_all(db),
    )?;
    Ok(AdminBundle { fields, courses, ebooks, other_courses, other_ebooks })
}

pub async fn admin_detail(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<Json<AdminBundle>> {
    Ok(Json(admin_load(&st.db, id).await?))
}

#[derive(Deserialize)]
pub struct BundleInput {
    slug: String,
    title: String,
    summary: Option<String>,
    #[serde(default)]
    description: String,
    cover_url: Option<String>,
    price_cents: i32,
    sale_price_cents: Option<i32>,
    #[serde(default)]
    published: bool,
}

struct BundleFields {
    slug: String,
    title: String,
    summary: Option<String>,
    description: String,
    cover_url: Option<String>,
    price_cents: i32,
    sale_price_cents: Option<i32>,
    published: bool,
}

impl BundleInput {
    fn validated(self) -> AppResult<BundleFields> {
        validate::prices(self.price_cents, self.sale_price_cents)?;
        let description = self.description.trim().to_owned();
        if description.chars().count() > 20_000 {
            return Err(AppError::Validation("invalid_description"));
        }
        Ok(BundleFields {
            slug: validate::slug(&self.slug)?,
            title: validate::required(&self.title, 200, "invalid_title")?,
            summary: validate::optional(self.summary.as_deref(), 500, "invalid_summary")?,
            description,
            cover_url: validate::optional_url(self.cover_url.as_deref())?,
            price_cents: self.price_cents,
            sale_price_cents: self.sale_price_cents,
            published: self.published,
        })
    }
}

pub async fn create(
    State(st): State<AppState>,
    _: AdminUser,
    Json(input): Json<BundleInput>,
) -> AppResult<(StatusCode, Json<AdminBundle>)> {
    let f = input.validated()?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO bundles (slug, title, summary, description, cover_url, price_cents, sale_price_cents, published)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(f.slug)
    .bind(f.title)
    .bind(f.summary)
    .bind(f.description)
    .bind(f.cover_url)
    .bind(f.price_cents)
    .bind(f.sale_price_cents)
    .bind(f.published)
    .fetch_one(&st.db)
    .await
    .map_err(validate::unique_violation("slug_taken"))?;
    Ok((StatusCode::CREATED, Json(admin_load(&st.db, id).await?)))
}

pub async fn update(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<BundleInput>,
) -> AppResult<Json<AdminBundle>> {
    let f = input.validated()?;
    let updated = sqlx::query(
        "UPDATE bundles SET slug = $2, title = $3, summary = $4, description = $5, cover_url = $6,
                price_cents = $7, sale_price_cents = $8, published = $9, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(f.slug)
    .bind(f.title)
    .bind(f.summary)
    .bind(f.description)
    .bind(f.cover_url)
    .bind(f.price_cents)
    .bind(f.sale_price_cents)
    .bind(f.published)
    .execute(&st.db)
    .await
    .map_err(validate::unique_violation("slug_taken"))?
    .rows_affected();
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(admin_load(&st.db, id).await?))
}

/// Refused with `409 has_buyers` once it was ordered (unpublish instead).
pub async fn delete(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let mut tx = st.db.begin().await?;
    let ordered: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM order_items WHERE bundle_id = $1)")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    if ordered {
        return Err(AppError::Conflict("has_buyers"));
    }
    let deleted = sqlx::query("DELETE FROM bundles WHERE id = $1").bind(id).execute(&mut *tx).await?.rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `courses` / `ebooks` → the bundle_items column and the item table.
fn item_column(kind: &str) -> AppResult<(&'static str, &'static str)> {
    match kind {
        "courses" => Ok(("course_id", "courses")),
        "ebooks" => Ok(("ebook_id", "ebooks")),
        _ => Err(AppError::NotFound),
    }
}

/// `PUT /api/admin/bundles/{id}/{courses|ebooks}/{item}`: appends an item (idempotent). Past buyers
/// keep what they were granted; new buyers get the current contents.
pub async fn add_item(
    State(st): State<AppState>,
    _: AdminUser,
    Path((id, kind, item)): Path<(Uuid, String, Uuid)>,
) -> AppResult<Json<AdminBundle>> {
    let (col, table) = item_column(&kind)?;
    let found: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS (SELECT 1 FROM bundles WHERE id = $1) AND EXISTS (SELECT 1 FROM {table} WHERE id = $2)"
    ))
    .bind(id)
    .bind(item)
    .fetch_one(&st.db)
    .await?;
    if !found {
        return Err(AppError::NotFound);
    }
    sqlx::query(&format!(
        "INSERT INTO bundle_items (bundle_id, {col}, position)
         SELECT $1, $2, COALESCE(MAX(position) + 1, 0) FROM bundle_items WHERE bundle_id = $1
         ON CONFLICT DO NOTHING"
    ))
    .bind(id)
    .bind(item)
    .execute(&st.db)
    .await?;
    Ok(Json(admin_load(&st.db, id).await?))
}

pub async fn remove_item(
    State(st): State<AppState>,
    _: AdminUser,
    Path((id, kind, item)): Path<(Uuid, String, Uuid)>,
) -> AppResult<Json<AdminBundle>> {
    let (col, _) = item_column(&kind)?;
    sqlx::query(&format!("DELETE FROM bundle_items WHERE bundle_id = $1 AND {col} = $2"))
        .bind(id)
        .bind(item)
        .execute(&st.db)
        .await?;
    Ok(Json(admin_load(&st.db, id).await?))
}
