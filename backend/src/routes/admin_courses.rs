//! Admin CRUD for courses and their curriculum (sections → lessons), plus publish toggles.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{AdminCourse, AdminCourseFields, AdminCourseRow, AdminLesson, AdminSection, ListParams, Page},
    validate,
};

const LIST_FROM: &str = r#"
FROM courses c
JOIN instructors i ON i.id = c.instructor_id
LEFT JOIN categories cat ON cat.id = c.category_id
WHERE ($1::text IS NULL OR c.title ILIKE $1 OR c.slug ILIKE $1 OR i.name ILIKE $1)
"#;

/// All courses, published or not, newest edit first. `?q=` searches title, slug, instructor.
pub async fn list(
    State(st): State<AppState>,
    _: AdminUser,
    Query(p): Query<ListParams>,
) -> AppResult<Json<Page<AdminCourseRow>>> {
    let pattern = p.search_pattern();
    let list_sql = format!(
        "SELECT c.id, c.slug, c.title, c.published, c.price_cents, c.sale_price_cents,
                i.name AS instructor_name, cat.name_km AS category_name_km,
                (SELECT COUNT(*) FROM lessons l JOIN course_sections s ON s.id = l.section_id
                 WHERE s.course_id = c.id) AS lesson_count,
                (SELECT COUNT(*) FROM enrollments en WHERE en.course_id = c.id) AS enrollment_count,
                c.updated_at
         {LIST_FROM} ORDER BY c.updated_at DESC LIMIT $2 OFFSET $3"
    );
    let count_sql = format!("SELECT COUNT(*) {LIST_FROM}");
    let (items, total) = tokio::try_join!(
        sqlx::query_as::<_, AdminCourseRow>(&list_sql)
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
pub struct CourseInput {
    slug: String,
    title: String,
    summary: Option<String>,
    #[serde(default)]
    description: String,
    cover_url: Option<String>,
    instructor_id: Uuid,
    category_id: Option<Uuid>,
    price_cents: i32,
    sale_price_cents: Option<i32>,
    #[serde(default)]
    published: bool,
}

struct CourseFields {
    slug: String,
    title: String,
    summary: Option<String>,
    description: String,
    cover_url: Option<String>,
    instructor_id: Uuid,
    category_id: Option<Uuid>,
    price_cents: i32,
    sale_price_cents: Option<i32>,
    published: bool,
}

impl CourseInput {
    fn validated(self) -> AppResult<CourseFields> {
        validate::prices(self.price_cents, self.sale_price_cents)?;
        Ok(CourseFields {
            slug: validate::slug(&self.slug)?,
            title: validate::required(&self.title, 200, "invalid_title")?,
            summary: validate::optional(self.summary.as_deref(), 500, "invalid_summary")?,
            description: validate::optional(Some(&self.description), 20_000, "invalid_description")?.unwrap_or_default(),
            cover_url: validate::optional_url(self.cover_url.as_deref())?,
            instructor_id: self.instructor_id,
            category_id: self.category_id,
            price_cents: self.price_cents,
            sale_price_cents: self.sale_price_cents,
            published: self.published,
        })
    }
}

/// Unknown instructor/category/media ids surface as `422 invalid_reference`.
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
    Json(input): Json<CourseInput>,
) -> AppResult<(StatusCode, Json<AdminCourse>)> {
    let f = input.validated()?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO courses (slug, title, summary, description, cover_url, instructor_id, category_id,
                              price_cents, sale_price_cents, published)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
    )
    .bind(f.slug)
    .bind(f.title)
    .bind(f.summary)
    .bind(f.description)
    .bind(f.cover_url)
    .bind(f.instructor_id)
    .bind(f.category_id)
    .bind(f.price_cents)
    .bind(f.sale_price_cents)
    .bind(f.published)
    .fetch_one(&st.db)
    .await
    .map_err(map_write_error)?;
    Ok((StatusCode::CREATED, Json(load(&st.db, id).await?)))
}

pub async fn detail(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<Json<AdminCourse>> {
    Ok(Json(load(&st.db, id).await?))
}

pub async fn update(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<CourseInput>,
) -> AppResult<Json<AdminCourse>> {
    let f = input.validated()?;
    let updated = sqlx::query(
        "UPDATE courses SET slug = $2, title = $3, summary = $4, description = $5, cover_url = $6,
                instructor_id = $7, category_id = $8, price_cents = $9, sale_price_cents = $10,
                published = $11, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(f.slug)
    .bind(f.title)
    .bind(f.summary)
    .bind(f.description)
    .bind(f.cover_url)
    .bind(f.instructor_id)
    .bind(f.category_id)
    .bind(f.price_cents)
    .bind(f.sale_price_cents)
    .bind(f.published)
    .execute(&st.db)
    .await
    .map_err(map_write_error)?
    .rows_affected();
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(load(&st.db, id).await?))
}

#[derive(Deserialize)]
pub struct PublishInput {
    pub(crate) published: bool,
}

pub async fn publish(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<PublishInput>,
) -> AppResult<StatusCode> {
    set_published(&st.db, "courses", id, input.published).await
}

/// `table` is always a literal (`courses` / `ebooks`).
pub(crate) async fn set_published(db: &PgPool, table: &'static str, id: Uuid, published: bool) -> AppResult<StatusCode> {
    let updated = sqlx::query(&format!("UPDATE {table} SET published = $2, updated_at = now() WHERE id = $1"))
        .bind(id)
        .bind(published)
        .execute(db)
        .await?
        .rows_affected();
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Refused with `409 has_buyers` once anyone enrolled or ordered it (deleting would revoke access
/// or orphan orders); unpublish instead.
pub async fn delete(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let mut tx = st.db.begin().await?;
    let has_buyers: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM enrollments WHERE course_id = $1)
             OR EXISTS (SELECT 1 FROM order_items WHERE course_id = $1)",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if has_buyers {
        return Err(AppError::Conflict("has_buyers"));
    }
    let deleted = sqlx::query("DELETE FROM courses WHERE id = $1").bind(id).execute(&mut *tx).await?.rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn load(db: &PgPool, id: Uuid) -> AppResult<AdminCourse> {
    let fields = sqlx::query_as::<_, AdminCourseFields>(
        "SELECT id, slug, title, summary, description, cover_url, instructor_id, category_id,
                price_cents, sale_price_cents, published, updated_at
         FROM courses WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)?;
    let (sections, lessons) = tokio::try_join!(
        sqlx::query_as::<_, (Uuid, String, i32)>(
            "SELECT id, title, position FROM course_sections WHERE course_id = $1 ORDER BY position",
        )
        .bind(id)
        .fetch_all(db),
        sqlx::query_as::<_, AdminLesson>(
            "SELECT l.id, l.section_id, l.title, l.position, l.video_id, l.media_id, m.file_name AS media_file_name,
                    l.duration_seconds, l.is_preview
             FROM lessons l JOIN course_sections s ON s.id = l.section_id
             LEFT JOIN media_files m ON m.id = l.media_id
             WHERE s.course_id = $1 ORDER BY l.position",
        )
        .bind(id)
        .fetch_all(db),
    )?;
    let mut sections: Vec<AdminSection> = sections
        .into_iter()
        .map(|(id, title, position)| AdminSection { id, title, position, lessons: Vec::new() })
        .collect();
    for lesson in lessons {
        if let Some(s) = sections.iter_mut().find(|s| s.id == lesson.section_id) {
            s.lessons.push(lesson);
        }
    }
    Ok(AdminCourse { fields, sections })
}

// ---- Sections ------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SectionInput {
    title: String,
}

/// Appends a section to the course.
pub async fn create_section(
    State(st): State<AppState>,
    _: AdminUser,
    Path(course_id): Path<Uuid>,
    Json(input): Json<SectionInput>,
) -> AppResult<(StatusCode, Json<AdminCourse>)> {
    let title = validate::required(&input.title, 200, "invalid_title")?;
    let mut tx = st.db.begin().await?;
    lock_course(&mut tx, course_id).await?;
    sqlx::query(
        "INSERT INTO course_sections (course_id, title, position)
         SELECT $1, $2, COALESCE(MAX(position), 0) + 1 FROM course_sections WHERE course_id = $1",
    )
    .bind(course_id)
    .bind(title)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(load(&st.db, course_id).await?)))
}

pub async fn update_section(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<SectionInput>,
) -> AppResult<Json<AdminCourse>> {
    let title = validate::required(&input.title, 200, "invalid_title")?;
    let course_id: Uuid =
        sqlx::query_scalar("UPDATE course_sections SET title = $2 WHERE id = $1 RETURNING course_id")
            .bind(id)
            .bind(title)
            .fetch_optional(&st.db)
            .await?
            .ok_or(AppError::NotFound)?;
    Ok(Json(load(&st.db, course_id).await?))
}

/// Deletes the section and its lessons (and learners' progress on them).
pub async fn delete_section(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<Json<AdminCourse>> {
    let course_id: Uuid = sqlx::query_scalar("DELETE FROM course_sections WHERE id = $1 RETURNING course_id")
        .bind(id)
        .fetch_optional(&st.db)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(load(&st.db, course_id).await?))
}

#[derive(Deserialize)]
pub struct MoveInput {
    /// `up` or `down`
    direction: String,
}

pub async fn move_section(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<MoveInput>,
) -> AppResult<Json<AdminCourse>> {
    let up = parse_direction(&input.direction)?;
    let mut tx = st.db.begin().await?;
    let course_id: Uuid = sqlx::query_scalar("SELECT course_id FROM course_sections WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;
    lock_course(&mut tx, course_id).await?;
    swap_with_neighbor(&mut tx, "course_sections", "course_id", id, up).await?;
    tx.commit().await?;
    Ok(Json(load(&st.db, course_id).await?))
}

// ---- Lessons -------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct LessonInput {
    title: String,
    video_id: Option<String>,
    /// Uploaded media (Telegram); wins over `video_id` when set.
    media_id: Option<Uuid>,
    #[serde(default)]
    duration_seconds: i32,
    #[serde(default)]
    is_preview: bool,
}

struct LessonFields {
    title: String,
    video_id: Option<String>,
    media_id: Option<Uuid>,
    duration_seconds: i32,
    is_preview: bool,
}

impl LessonInput {
    fn validated(self) -> AppResult<LessonFields> {
        if self.duration_seconds < 0 {
            return Err(AppError::Validation("invalid_duration"));
        }
        Ok(LessonFields {
            title: validate::required(&self.title, 200, "invalid_title")?,
            video_id: validate::optional(self.video_id.as_deref(), 200, "invalid_video_id")?,
            media_id: self.media_id,
            duration_seconds: self.duration_seconds,
            is_preview: self.is_preview,
        })
    }
}

/// Appends a lesson to the section.
pub async fn create_lesson(
    State(st): State<AppState>,
    _: AdminUser,
    Path(section_id): Path<Uuid>,
    Json(input): Json<LessonInput>,
) -> AppResult<(StatusCode, Json<AdminCourse>)> {
    let f = input.validated()?;
    let mut tx = st.db.begin().await?;
    let course_id = section_course(&mut tx, section_id).await?;
    lock_course(&mut tx, course_id).await?;
    sqlx::query(
        "INSERT INTO lessons (section_id, title, position, video_id, media_id, duration_seconds, is_preview)
         SELECT $1, $2, COALESCE(MAX(position), 0) + 1, $3, $4, $5, $6 FROM lessons WHERE section_id = $1",
    )
    .bind(section_id)
    .bind(f.title)
    .bind(f.video_id)
    .bind(f.media_id)
    .bind(f.duration_seconds)
    .bind(f.is_preview)
    .execute(&mut *tx)
    .await
    .map_err(map_write_error)?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(load(&st.db, course_id).await?)))
}

pub async fn update_lesson(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<LessonInput>,
) -> AppResult<Json<AdminCourse>> {
    let f = input.validated()?;
    let course_id: Uuid = sqlx::query_scalar(
        "UPDATE lessons l SET title = $2, video_id = $3, media_id = $4, duration_seconds = $5, is_preview = $6
         FROM course_sections s WHERE l.id = $1 AND s.id = l.section_id
         RETURNING s.course_id",
    )
    .bind(id)
    .bind(f.title)
    .bind(f.video_id)
    .bind(f.media_id)
    .bind(f.duration_seconds)
    .bind(f.is_preview)
    .fetch_optional(&st.db)
    .await
    .map_err(map_write_error)?
    .ok_or(AppError::NotFound)?;
    Ok(Json(load(&st.db, course_id).await?))
}

pub async fn delete_lesson(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<Json<AdminCourse>> {
    let course_id: Uuid = sqlx::query_scalar(
        "DELETE FROM lessons l USING course_sections s WHERE l.id = $1 AND s.id = l.section_id RETURNING s.course_id",
    )
    .bind(id)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(load(&st.db, course_id).await?))
}

pub async fn move_lesson(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<MoveInput>,
) -> AppResult<Json<AdminCourse>> {
    let up = parse_direction(&input.direction)?;
    let mut tx = st.db.begin().await?;
    let section_id: Uuid = sqlx::query_scalar("SELECT section_id FROM lessons WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;
    let course_id = section_course(&mut tx, section_id).await?;
    lock_course(&mut tx, course_id).await?;
    swap_with_neighbor(&mut tx, "lessons", "section_id", id, up).await?;
    tx.commit().await?;
    Ok(Json(load(&st.db, course_id).await?))
}

// ---- Helpers -------------------------------------------------------------------------------

fn parse_direction(d: &str) -> AppResult<bool> {
    match d {
        "up" => Ok(true),
        "down" => Ok(false),
        _ => Err(AppError::Validation("invalid_direction")),
    }
}

/// Serializes curriculum edits per course so appends/moves don't race on `position`.
async fn lock_course(tx: &mut PgConnection, course_id: Uuid) -> AppResult<()> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM courses WHERE id = $1 FOR UPDATE")
        .bind(course_id)
        .fetch_optional(tx)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(())
}

async fn section_course(tx: &mut PgConnection, section_id: Uuid) -> AppResult<Uuid> {
    sqlx::query_scalar("SELECT course_id FROM course_sections WHERE id = $1")
        .bind(section_id)
        .fetch_optional(tx)
        .await?
        .ok_or(AppError::NotFound)
}

/// Swaps `id`'s position with the previous (`up`) or next sibling under the same parent; no-op at
/// the edge. Goes through a temporary position because `(parent, position)` is unique.
/// `table` / `parent` are always literals from this module.
async fn swap_with_neighbor(
    tx: &mut PgConnection,
    table: &'static str,
    parent: &'static str,
    id: Uuid,
    up: bool,
) -> AppResult<()> {
    let (cmp, order) = if up { ("<", "DESC") } else { (">", "ASC") };
    let neighbor: Option<(Uuid, i32, i32)> = sqlx::query_as(&format!(
        "SELECT n.id, n.position, me.position FROM {table} me
         JOIN {table} n ON n.{parent} = me.{parent} AND n.position {cmp} me.position
         WHERE me.id = $1 ORDER BY n.position {order} LIMIT 1"
    ))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((other, other_pos, my_pos)) = neighbor else { return Ok(()) };
    for (row, pos) in [(id, -1), (other, my_pos), (id, other_pos)] {
        sqlx::query(&format!("UPDATE {table} SET position = $2 WHERE id = $1"))
            .bind(row)
            .bind(pos)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}
