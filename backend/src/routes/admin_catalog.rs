//! Admin CRUD for the small catalog tables: categories, instructors, banners.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{AdminBanner, AdminCategory, AdminInstructor},
    validate,
};

// ---- Categories ----------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CategoryInput {
    slug: String,
    name_km: String,
    name_en: String,
    #[serde(default)]
    position: i32,
}

impl CategoryInput {
    fn validated(self) -> AppResult<(String, String, String, i32)> {
        Ok((
            validate::slug(&self.slug)?,
            validate::required(&self.name_km, 100, "invalid_name")?,
            validate::required(&self.name_en, 100, "invalid_name")?,
            self.position,
        ))
    }
}

const CATEGORY_COLUMNS: &str = "id, slug, name_km, name_en, position";

pub async fn list_categories(State(st): State<AppState>, _: AdminUser) -> AppResult<Json<Vec<AdminCategory>>> {
    let rows = sqlx::query_as::<_, AdminCategory>(&format!(
        "SELECT {CATEGORY_COLUMNS} FROM categories ORDER BY position, name_en"
    ))
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}

pub async fn create_category(
    State(st): State<AppState>,
    _: AdminUser,
    Json(input): Json<CategoryInput>,
) -> AppResult<(StatusCode, Json<AdminCategory>)> {
    let (slug, name_km, name_en, position) = input.validated()?;
    let row = sqlx::query_as::<_, AdminCategory>(&format!(
        "INSERT INTO categories (slug, name_km, name_en, position) VALUES ($1, $2, $3, $4) RETURNING {CATEGORY_COLUMNS}"
    ))
    .bind(slug)
    .bind(name_km)
    .bind(name_en)
    .bind(position)
    .fetch_one(&st.db)
    .await
    .map_err(validate::unique_violation("slug_taken"))?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn update_category(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<CategoryInput>,
) -> AppResult<Json<AdminCategory>> {
    let (slug, name_km, name_en, position) = input.validated()?;
    let row = sqlx::query_as::<_, AdminCategory>(&format!(
        "UPDATE categories SET slug = $2, name_km = $3, name_en = $4, position = $5 WHERE id = $1 \
         RETURNING {CATEGORY_COLUMNS}"
    ))
    .bind(id)
    .bind(slug)
    .bind(name_km)
    .bind(name_en)
    .bind(position)
    .fetch_optional(&st.db)
    .await
    .map_err(validate::unique_violation("slug_taken"))?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

/// Courses and ebooks in the category become uncategorized.
pub async fn delete_category(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    delete_by_id(&st, "categories", id).await
}

// ---- Instructors ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct InstructorInput {
    name: String,
    bio: Option<String>,
    avatar_url: Option<String>,
}

impl InstructorInput {
    fn validated(self) -> AppResult<(String, Option<String>, Option<String>)> {
        Ok((
            validate::required(&self.name, 100, "invalid_name")?,
            validate::optional(self.bio.as_deref(), 2000, "invalid_bio")?,
            validate::optional_url(self.avatar_url.as_deref())?,
        ))
    }
}

const INSTRUCTOR_SELECT: &str = "SELECT i.id, i.name, i.bio, i.avatar_url, \
     (SELECT COUNT(*) FROM courses c WHERE c.instructor_id = i.id) AS course_count FROM instructors i";

pub async fn list_instructors(State(st): State<AppState>, _: AdminUser) -> AppResult<Json<Vec<AdminInstructor>>> {
    let rows = sqlx::query_as::<_, AdminInstructor>(&format!("{INSTRUCTOR_SELECT} ORDER BY i.name"))
        .fetch_all(&st.db)
        .await?;
    Ok(Json(rows))
}

pub async fn create_instructor(
    State(st): State<AppState>,
    _: AdminUser,
    Json(input): Json<InstructorInput>,
) -> AppResult<(StatusCode, Json<AdminInstructor>)> {
    let (name, bio, avatar_url) = input.validated()?;
    let id: Uuid = sqlx::query_scalar("INSERT INTO instructors (name, bio, avatar_url) VALUES ($1, $2, $3) RETURNING id")
        .bind(name)
        .bind(bio)
        .bind(avatar_url)
        .fetch_one(&st.db)
        .await?;
    Ok((StatusCode::CREATED, Json(load_instructor(&st, id).await?)))
}

pub async fn update_instructor(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<InstructorInput>,
) -> AppResult<Json<AdminInstructor>> {
    let (name, bio, avatar_url) = input.validated()?;
    let updated = sqlx::query("UPDATE instructors SET name = $2, bio = $3, avatar_url = $4 WHERE id = $1")
        .bind(id)
        .bind(name)
        .bind(bio)
        .bind(avatar_url)
        .execute(&st.db)
        .await?
        .rows_affected();
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(load_instructor(&st, id).await?))
}

/// 409 `in_use` while any course references the instructor.
pub async fn delete_instructor(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    delete_by_id(&st, "instructors", id).await
}

async fn load_instructor(st: &AppState, id: Uuid) -> AppResult<AdminInstructor> {
    sqlx::query_as::<_, AdminInstructor>(&format!("{INSTRUCTOR_SELECT} WHERE i.id = $1"))
        .bind(id)
        .fetch_optional(&st.db)
        .await?
        .ok_or(AppError::NotFound)
}

// ---- Banners -------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct BannerInput {
    title: String,
    subtitle: Option<String>,
    image_url: Option<String>,
    link_url: Option<String>,
    #[serde(default)]
    position: i32,
    #[serde(default)]
    published: bool,
}

const BANNER_COLUMNS: &str = "id, title, subtitle, image_url, link_url, position, published";

type BannerFields = (String, Option<String>, Option<String>, Option<String>, i32, bool);

impl BannerInput {
    fn validated(self) -> AppResult<BannerFields> {
        Ok((
            validate::required(&self.title, 200, "invalid_title")?,
            validate::optional(self.subtitle.as_deref(), 500, "invalid_subtitle")?,
            validate::optional_url(self.image_url.as_deref())?,
            validate::optional_url(self.link_url.as_deref())?,
            self.position,
            self.published,
        ))
    }
}

pub async fn list_banners(State(st): State<AppState>, _: AdminUser) -> AppResult<Json<Vec<AdminBanner>>> {
    let rows = sqlx::query_as::<_, AdminBanner>(&format!(
        "SELECT {BANNER_COLUMNS} FROM banners ORDER BY position, created_at"
    ))
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}

pub async fn create_banner(
    State(st): State<AppState>,
    _: AdminUser,
    Json(input): Json<BannerInput>,
) -> AppResult<(StatusCode, Json<AdminBanner>)> {
    let (title, subtitle, image_url, link_url, position, published) = input.validated()?;
    let row = sqlx::query_as::<_, AdminBanner>(&format!(
        "INSERT INTO banners (title, subtitle, image_url, link_url, position, published) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING {BANNER_COLUMNS}"
    ))
    .bind(title)
    .bind(subtitle)
    .bind(image_url)
    .bind(link_url)
    .bind(position)
    .bind(published)
    .fetch_one(&st.db)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn update_banner(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<BannerInput>,
) -> AppResult<Json<AdminBanner>> {
    let (title, subtitle, image_url, link_url, position, published) = input.validated()?;
    let row = sqlx::query_as::<_, AdminBanner>(&format!(
        "UPDATE banners SET title = $2, subtitle = $3, image_url = $4, link_url = $5, position = $6, \
         published = $7 WHERE id = $1 RETURNING {BANNER_COLUMNS}"
    ))
    .bind(id)
    .bind(title)
    .bind(subtitle)
    .bind(image_url)
    .bind(link_url)
    .bind(position)
    .bind(published)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

pub async fn delete_banner(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    delete_by_id(&st, "banners", id).await
}

/// `table` is always a literal from this module.
async fn delete_by_id(st: &AppState, table: &'static str, id: Uuid) -> AppResult<StatusCode> {
    let deleted = sqlx::query(&format!("DELETE FROM {table} WHERE id = $1"))
        .bind(id)
        .execute(&st.db)
        .await
        .map_err(validate::still_referenced)?
        .rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
