//! Lesson Q&A. Whoever may watch a lesson may read its questions; enrolled learners ask; admins
//! and the course's own instructor (`instructors.user_id`) answer. Authors and staff delete.

use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::lessons::{LessonGate, check_access, load_lesson};
use crate::{
    AppState,
    auth::{AdminUser, CurrentUser},
    error::{AppError, AppResult},
    models::{AdminQuestion, Answer, LessonQuestions, ListParams, Page, Question, QuestionRow},
    validate,
};

#[derive(Deserialize)]
pub struct BodyInput {
    body: String,
}

/// Admin, or the user linked to the course's instructor.
async fn is_staff(db: &PgPool, current: &CurrentUser, course_id: Uuid) -> sqlx::Result<bool> {
    if current.user.role == "admin" {
        return Ok(true);
    }
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM courses c JOIN instructors i ON i.id = c.instructor_id \
         WHERE c.id = $1 AND i.user_id = $2)",
    )
    .bind(course_id)
    .bind(current.user.id)
    .fetch_one(db)
    .await
}

/// Latest 100 questions on a lesson with their answers (oldest answer first).
async fn load(db: &PgPool, lesson_id: Uuid, viewer: Option<Uuid>, can_answer: bool) -> AppResult<LessonQuestions> {
    let rows = sqlx::query_as::<_, QuestionRow>(
        "SELECT q.id, u.full_name AS user_name, q.body, q.created_at, q.user_id = $2 IS TRUE AS mine
         FROM lesson_questions q JOIN users u ON u.id = q.user_id
         WHERE q.lesson_id = $1 ORDER BY q.created_at DESC LIMIT 100",
    )
    .bind(lesson_id)
    .bind(viewer)
    .fetch_all(db)
    .await?;
    let ids: Vec<Uuid> = rows.iter().map(|q| q.id).collect();
    let mut answers: HashMap<Uuid, Vec<Answer>> = HashMap::new();
    for a in sqlx::query_as::<_, Answer>(
        "SELECT a.id, a.question_id, u.full_name AS user_name, a.body, a.created_at, a.user_id = $2 IS TRUE AS mine
         FROM lesson_answers a JOIN users u ON u.id = a.user_id
         WHERE a.question_id = ANY($1) ORDER BY a.created_at",
    )
    .bind(&ids)
    .bind(viewer)
    .fetch_all(db)
    .await?
    {
        answers.entry(a.question_id).or_default().push(a);
    }
    let questions = rows
        .into_iter()
        .map(|q| Question { answers: answers.remove(&q.id).unwrap_or_default(), row: q })
        .collect();
    Ok(LessonQuestions { questions, can_answer })
}

/// `GET /api/courses/{slug}/lessons/{id}/questions`: same access as the lesson itself.
pub async fn list(
    State(st): State<AppState>,
    current: Option<CurrentUser>,
    Path((slug, id)): Path<(String, Uuid)>,
) -> AppResult<Json<LessonQuestions>> {
    let row = load_lesson(&st, &slug, id, current.as_ref()).await?;
    let staff = match &current {
        Some(c) => is_staff(&st.db, c, row.course_id).await?,
        None => false,
    };
    if !staff {
        check_access(LessonGate {
            published: row.published,
            is_preview: row.is_preview,
            signed_in: current.is_some(),
            enrolled: row.enrolled,
        })?;
    }
    Ok(Json(load(&st.db, id, current.map(|c| c.user.id), staff).await?))
}

/// Enrolled learners (and staff) ask; 403 otherwise, 422 `invalid_body`.
pub async fn ask(
    State(st): State<AppState>,
    current: CurrentUser,
    Path((slug, id)): Path<(String, Uuid)>,
    Json(input): Json<BodyInput>,
) -> AppResult<(StatusCode, Json<LessonQuestions>)> {
    let body = validate::required(&input.body, 2000, "invalid_body")?;
    let row = load_lesson(&st, &slug, id, Some(&current)).await?;
    let staff = is_staff(&st.db, &current, row.course_id).await?;
    if !(row.enrolled || staff) {
        return Err(AppError::Forbidden);
    }
    sqlx::query("INSERT INTO lesson_questions (lesson_id, user_id, body) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(current.user.id)
        .bind(body)
        .execute(&st.db)
        .await?;
    Ok((StatusCode::CREATED, Json(load(&st.db, id, Some(current.user.id), staff).await?)))
}

/// The lesson and course a question belongs to, and its author.
async fn question_context(db: &PgPool, question_id: Uuid) -> AppResult<(Uuid, Uuid, Uuid)> {
    sqlx::query_as::<_, (Uuid, Uuid, Uuid)>(
        "SELECT q.lesson_id, s.course_id, q.user_id FROM lesson_questions q
         JOIN lessons l ON l.id = q.lesson_id JOIN course_sections s ON s.id = l.section_id
         WHERE q.id = $1",
    )
    .bind(question_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)
}

/// `POST /api/questions/{id}/answers`: staff only → the lesson's questions.
pub async fn answer(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(question_id): Path<Uuid>,
    Json(input): Json<BodyInput>,
) -> AppResult<(StatusCode, Json<LessonQuestions>)> {
    let body = validate::required(&input.body, 4000, "invalid_body")?;
    let (lesson_id, course_id, _) = question_context(&st.db, question_id).await?;
    if !is_staff(&st.db, &current, course_id).await? {
        return Err(AppError::Forbidden);
    }
    sqlx::query("INSERT INTO lesson_answers (question_id, user_id, body) VALUES ($1, $2, $3)")
        .bind(question_id)
        .bind(current.user.id)
        .bind(body)
        .execute(&st.db)
        .await?;
    Ok((StatusCode::CREATED, Json(load(&st.db, lesson_id, Some(current.user.id), true).await?)))
}

/// The author or staff delete a question (with its answers).
pub async fn delete_question(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(question_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let (_, course_id, author) = question_context(&st.db, question_id).await?;
    if author != current.user.id && !is_staff(&st.db, &current, course_id).await? {
        return Err(AppError::Forbidden);
    }
    sqlx::query("DELETE FROM lesson_questions WHERE id = $1").bind(question_id).execute(&st.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The author or staff delete an answer.
pub async fn delete_answer(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(answer_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let (question_id, author): (Uuid, Uuid) =
        sqlx::query_as("SELECT question_id, user_id FROM lesson_answers WHERE id = $1")
            .bind(answer_id)
            .fetch_optional(&st.db)
            .await?
            .ok_or(AppError::NotFound)?;
    let (_, course_id, _) = question_context(&st.db, question_id).await?;
    if author != current.user.id && !is_staff(&st.db, &current, course_id).await? {
        return Err(AppError::Forbidden);
    }
    sqlx::query("DELETE FROM lesson_answers WHERE id = $1").bind(answer_id).execute(&st.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct AdminQuestionParams {
    /// `true` (default): only questions without an answer, oldest first.
    unanswered: Option<bool>,
}

const ADMIN_FROM: &str = "FROM lesson_questions q
     JOIN users u ON u.id = q.user_id
     JOIN lessons l ON l.id = q.lesson_id
     JOIN course_sections s ON s.id = l.section_id
     JOIN courses c ON c.id = s.course_id
     WHERE (NOT $1 OR NOT EXISTS (SELECT 1 FROM lesson_answers a WHERE a.question_id = q.id))";

/// `GET /api/admin/questions?unanswered=&page=`: the answer queue across all courses.
pub async fn admin_list(
    State(st): State<AppState>,
    _: AdminUser,
    Query(p): Query<ListParams>,
    Query(ap): Query<AdminQuestionParams>,
) -> AppResult<Json<Page<AdminQuestion>>> {
    let unanswered = ap.unanswered.unwrap_or(true);
    let order = if unanswered { "ASC" } else { "DESC" };
    let list_sql = format!(
        "SELECT q.id, q.body, q.created_at, u.full_name AS user_name, c.slug AS course_slug, c.title AS course_title,
                l.id AS lesson_id, l.title AS lesson_title,
                (SELECT COUNT(*) FROM lesson_answers a WHERE a.question_id = q.id) AS answer_count
         {ADMIN_FROM} ORDER BY q.created_at {order} LIMIT $2 OFFSET $3"
    );
    let count_sql = format!("SELECT COUNT(*) {ADMIN_FROM}");
    let (items, total) = tokio::try_join!(
        sqlx::query_as::<_, AdminQuestion>(&list_sql)
            .bind(unanswered)
            .bind(p.per_page())
            .bind(p.offset())
            .fetch_all(&st.db),
        sqlx::query_scalar::<_, i64>(&count_sql).bind(unanswered).fetch_one(&st.db),
    )?;
    Ok(Json(Page { items, total, page: p.page(), per_page: p.per_page() }))
}
