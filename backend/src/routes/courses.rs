use axum::{
    Json,
    extract::{Path, Query, State},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, AppResult},
    models::*,
};

const CARD_SELECT: &str = r#"
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
    Path(slug): Path<String>,
) -> AppResult<Json<CourseDetail>> {
    let card = sqlx::query_as::<_, CourseCard>(&format!(
        "{CARD_SELECT} WHERE c.published AND c.slug = $1"
    ))
    .bind(&slug)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let (extra, section_rows, lesson_rows) = tokio::try_join!(
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
    )?;

    let lesson_count = lesson_rows.len();
    let total_duration_seconds = lesson_rows.iter().map(|l| i64::from(l.duration_seconds)).sum();
    let sections = group_lessons(section_rows, lesson_rows);

    Ok(Json(CourseDetail { card, extra, sections, lesson_count, total_duration_seconds }))
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
