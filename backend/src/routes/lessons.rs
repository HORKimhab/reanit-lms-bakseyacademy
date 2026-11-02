use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::{LessonAccessRow, LessonPlayback},
};

/// How long a signed embed URL stays valid.
const VIDEO_URL_TTL: Duration = Duration::hours(4);

/// Who may watch a lesson: enrolled users (even after the course is unpublished), and anyone
/// for preview lessons of a published course.
fn check_access(published: bool, is_preview: bool, signed_in: bool, enrolled: bool) -> AppResult<()> {
    if enrolled || (published && is_preview) {
        Ok(())
    } else if !published {
        Err(AppError::NotFound)
    } else if !signed_in {
        Err(AppError::Unauthorized("unauthorized"))
    } else {
        Err(AppError::Forbidden)
    }
}

pub async fn lesson(
    State(st): State<AppState>,
    current: Option<CurrentUser>,
    Path((slug, id)): Path<(String, Uuid)>,
) -> AppResult<Json<LessonPlayback>> {
    let row = sqlx::query_as::<_, LessonAccessRow>(
        "SELECT l.id, l.section_id, l.title, l.duration_seconds, l.is_preview, l.video_id, c.published,
                EXISTS (SELECT 1 FROM enrollments e WHERE e.course_id = c.id AND e.user_id = $3) AS enrolled
         FROM lessons l
         JOIN course_sections s ON s.id = l.section_id
         JOIN courses c ON c.id = s.course_id
         WHERE c.slug = $1 AND l.id = $2",
    )
    .bind(&slug)
    .bind(id)
    .bind(current.as_ref().map(|c| c.user.id))
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    check_access(row.published, row.is_preview, current.is_some(), row.enrolled)?;

    let expires = Utc::now() + VIDEO_URL_TTL;
    let video = st.video.as_ref().zip(row.video_id.as_deref());
    let video_url = video.map(|(bunny, vid)| bunny.embed_url(vid, expires.timestamp()));
    let expires_at = video.and_then(|(bunny, _)| bunny.token_key.as_ref()).map(|_| expires);

    Ok(Json(LessonPlayback {
        id: row.id,
        section_id: row.section_id,
        title: row.title,
        duration_seconds: row.duration_seconds,
        is_preview: row.is_preview,
        video_url,
        expires_at,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(r: AppResult<()>) -> &'static str {
        match r {
            Ok(()) => "ok",
            Err(AppError::NotFound) => "not_found",
            Err(AppError::Unauthorized(_)) => "unauthorized",
            Err(AppError::Forbidden) => "forbidden",
            Err(_) => "other",
        }
    }

    #[test]
    fn enrolled_users_watch_any_lesson_even_unpublished() {
        assert_eq!(code(check_access(true, false, true, true)), "ok");
        assert_eq!(code(check_access(false, false, true, true)), "ok");
    }

    #[test]
    fn previews_are_public_on_published_courses_only() {
        assert_eq!(code(check_access(true, true, false, false)), "ok");
        assert_eq!(code(check_access(false, true, false, false)), "not_found");
    }

    #[test]
    fn paid_lessons_need_sign_in_then_enrollment() {
        assert_eq!(code(check_access(true, false, false, false)), "unauthorized");
        assert_eq!(code(check_access(true, false, true, false)), "forbidden");
        assert_eq!(code(check_access(false, false, true, false)), "not_found");
    }
}
