//! Course completion certificates.

use axum::{
    Json,
    extract::{Path, State},
};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, AppResult},
    models::Certificate,
};

/// Issues the certificate once the enrolled user has completed every lesson of a course with at
/// least one lesson. Idempotent; an issued certificate stays when lessons are later added or
/// unmarked.
pub(crate) async fn issue_if_complete(db: impl PgExecutor<'_>, user_id: Uuid, course_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO certificates (user_id, course_id, full_name, course_title, instructor_name)
         SELECT u.id, c.id, u.full_name, c.title, i.name
         FROM users u, courses c JOIN instructors i ON i.id = c.instructor_id
         WHERE u.id = $1 AND c.id = $2
           AND EXISTS (SELECT 1 FROM enrollments en WHERE en.user_id = u.id AND en.course_id = c.id)
           AND EXISTS (SELECT 1 FROM lessons l JOIN course_sections s ON s.id = l.section_id WHERE s.course_id = c.id)
           AND NOT EXISTS (
               SELECT 1 FROM lessons l JOIN course_sections s ON s.id = l.section_id
               WHERE s.course_id = c.id
                 AND NOT EXISTS (SELECT 1 FROM lesson_progress p WHERE p.lesson_id = l.id AND p.user_id = u.id))
         ON CONFLICT (user_id, course_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(course_id)
    .execute(db)
    .await?;
    Ok(())
}

/// Public: anyone with the link can verify a certificate.
pub async fn detail(State(st): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<Certificate>> {
    let cert = sqlx::query_as::<_, Certificate>(
        "SELECT ce.id, ce.full_name, ce.course_title, ce.instructor_name, ce.issued_at, c.slug AS course_slug
         FROM certificates ce JOIN courses c ON c.id = ce.course_id WHERE ce.id = $1",
    )
    .bind(id)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(cert))
}
