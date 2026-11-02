use axum::{Json, extract::State};

use super::{courses, ebooks};
use crate::{
    AppState,
    auth::CurrentUser,
    error::AppResult,
    models::{CourseCard, EbookCard, Library},
};

/// Everything the user owns, most recently acquired first. Includes items unpublished since purchase.
pub async fn library(State(st): State<AppState>, current: CurrentUser) -> AppResult<Json<Library>> {
    let course_sql = format!(
        "{} JOIN enrollments en ON en.course_id = c.id AND en.user_id = $1 ORDER BY en.created_at DESC",
        courses::CARD_SELECT
    );
    let ebook_sql = format!(
        "{} JOIN ebook_purchases ep ON ep.ebook_id = e.id AND ep.user_id = $1 ORDER BY ep.created_at DESC",
        ebooks::CARD_SELECT
    );
    let (courses, ebooks) = tokio::try_join!(
        sqlx::query_as::<_, CourseCard>(&course_sql)
            .bind(current.user.id)
            .fetch_all(&st.db),
        sqlx::query_as::<_, EbookCard>(&ebook_sql)
            .bind(current.user.id)
            .fetch_all(&st.db),
    )?;
    Ok(Json(Library { courses, ebooks }))
}
