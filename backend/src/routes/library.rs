use std::collections::HashMap;

use axum::{Json, extract::State};
use uuid::Uuid;

use super::{courses, ebooks};
use crate::{
    AppState,
    auth::CurrentUser,
    error::AppResult,
    models::{CourseCard, EbookCard, Library, LibraryCourse},
};

/// Everything the user owns, most recently acquired first, with course progress.
/// Includes items unpublished since purchase.
pub async fn library(State(st): State<AppState>, current: CurrentUser) -> AppResult<Json<Library>> {
    let course_sql = format!(
        "{} JOIN enrollments en ON en.course_id = c.id AND en.user_id = $1 ORDER BY en.created_at DESC",
        courses::CARD_SELECT
    );
    let ebook_sql = format!(
        "{} JOIN ebook_purchases ep ON ep.ebook_id = e.id AND ep.user_id = $1 ORDER BY ep.created_at DESC",
        ebooks::CARD_SELECT
    );
    let (cards, progress, ebooks) = tokio::try_join!(
        sqlx::query_as::<_, CourseCard>(&course_sql)
            .bind(current.user.id)
            .fetch_all(&st.db),
        sqlx::query_as::<_, (Uuid, i64, i64)>(
            "SELECT s.course_id, COUNT(l.id), COUNT(p.lesson_id)
             FROM enrollments en
             JOIN course_sections s ON s.course_id = en.course_id
             JOIN lessons l ON l.section_id = s.id
             LEFT JOIN lesson_progress p ON p.lesson_id = l.id AND p.user_id = en.user_id
             WHERE en.user_id = $1
             GROUP BY s.course_id",
        )
        .bind(current.user.id)
        .fetch_all(&st.db),
        sqlx::query_as::<_, EbookCard>(&ebook_sql)
            .bind(current.user.id)
            .fetch_all(&st.db),
    )?;
    let progress: HashMap<Uuid, (i64, i64)> =
        progress.into_iter().map(|(id, total, done)| (id, (total, done))).collect();
    let courses = cards
        .into_iter()
        .map(|card| {
            let (lesson_count, completed_lessons) = progress.get(&card.id).copied().unwrap_or((0, 0));
            LibraryCourse { card, lesson_count, completed_lessons }
        })
        .collect();
    Ok(Json(Library { courses, ebooks }))
}
