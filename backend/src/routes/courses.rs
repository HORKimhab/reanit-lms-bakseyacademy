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
SELECT c.id, c.slug, c.title, c.summary, c.cover_url, c.price_cents, c.sale_price_cents,
       cat.slug AS category_slug, cat.name_km AS category_name_km,
       i.name AS instructor_name,
       COALESCE(r.avg, 0)::float8 AS rating_avg,
       COALESCE(r.cnt, 0)::int8 AS rating_count
FROM courses c
JOIN instructors i ON i.id = c.instructor_id
LEFT JOIN categories cat ON cat.id = c.category_id
LEFT JOIN LATERAL (
    SELECT AVG(rating) AS avg, COUNT(*) AS cnt FROM reviews WHERE course_id = c.id
) r ON true
"#;

const FILTER: &str = r#"
WHERE c.published
  AND ($1::text IS NULL OR c.title ILIKE $1 OR i.name ILIKE $1)
  AND ($2::text IS NULL OR cat.slug = $2)
"#;

pub async fn query_cards(
    db: &PgPool,
    pattern: Option<&str>,
    category: Option<&str>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<CourseCard>> {
    sqlx::query_as::<_, CourseCard>(&format!(
        "{CARD_SELECT} {FILTER} ORDER BY c.created_at DESC LIMIT $3 OFFSET $4"
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
) -> AppResult<Json<Page<CourseCard>>> {
    let pattern = p.search_pattern();
    let category = p.category();

    let count_sql = format!(
        "SELECT COUNT(*) FROM courses c
         JOIN instructors i ON i.id = c.instructor_id
         LEFT JOIN categories cat ON cat.id = c.category_id {FILTER}"
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
) -> AppResult<Json<CourseDetail>> {
    // Enrolled users keep seeing (and learning) a course after it is unpublished.
    let card = sqlx::query_as::<_, CourseCard>(&format!(
        "{CARD_SELECT} WHERE c.slug = $1 AND (c.published OR EXISTS (
             SELECT 1 FROM enrollments en WHERE en.course_id = c.id AND en.user_id = $2))"
    ))
    .bind(&slug)
    .bind(current.as_ref().map(|c| c.user.id))
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let (extra, section_rows, lesson_rows, owned) = tokio::try_join!(
        sqlx::query_as::<_, CourseExtra>(
            "SELECT c.description, i.bio AS instructor_bio, i.avatar_url AS instructor_avatar_url
             FROM courses c JOIN instructors i ON i.id = c.instructor_id
             WHERE c.id = $1",
        )
        .bind(card.id)
        .fetch_one(&st.db),
        sqlx::query_as::<_, SectionRow>(
            "SELECT id, title FROM course_sections WHERE course_id = $1 ORDER BY position",
        )
        .bind(card.id)
        .fetch_all(&st.db),
        sqlx::query_as::<_, LessonRow>(
            "SELECT l.id, l.section_id, l.title, l.duration_seconds, l.is_preview, l.video_id
             FROM lessons l JOIN course_sections s ON s.id = l.section_id
             WHERE s.course_id = $1
             ORDER BY s.position, l.position",
        )
        .bind(card.id)
        .fetch_all(&st.db),
        owns(&st.db, current.as_ref(), card.id),
    )?;

    let lesson_count = lesson_rows.len();
    let total_duration_seconds = lesson_rows.iter().map(|l| i64::from(l.duration_seconds)).sum();
    let sections = group_lessons(section_rows, lesson_rows);

    Ok(Json(CourseDetail { card, extra, sections, lesson_count, total_duration_seconds, owned }))
}

fn group_lessons(sections: Vec<SectionRow>, lessons: Vec<LessonRow>) -> Vec<SectionOut> {
    let mut out: Vec<SectionOut> = sections
        .into_iter()
        .map(|s| SectionOut { id: s.id, title: s.title, lessons: Vec::new() })
        .collect();
    let index = |id: Uuid, out: &[SectionOut]| out.iter().position(|s| s.id == id);
    for l in lessons {
        if let Some(i) = index(l.section_id, &out) {
            out[i].lessons.push(LessonOut {
                id: l.id,
                title: l.title,
                duration_seconds: l.duration_seconds,
                is_preview: l.is_preview,
                preview_video_id: if l.is_preview { l.video_id } else { None },
            });
        }
    }
    out
}

/// Whether the signed-in user (if any) owns this course.
async fn owns(db: &PgPool, current: Option<&CurrentUser>, id: Uuid) -> sqlx::Result<bool> {
    let Some(current) = current else { return Ok(false) };
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM enrollments WHERE user_id = $1 AND course_id = $2)")
        .bind(current.user.id)
        .bind(id)
        .fetch_one(db)
        .await
}
