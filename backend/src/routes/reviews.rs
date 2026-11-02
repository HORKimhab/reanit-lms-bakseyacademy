//! Reviews: owners rate (1–5) and optionally comment on a course or ebook; one review each.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::Review,
};

/// What a review is about. Table/column names are fixed strings, never user input.
#[derive(Clone, Copy)]
pub enum Target {
    Course,
    Ebook,
}

impl Target {
    const fn table(self) -> &'static str {
        match self {
            Target::Course => "courses",
            Target::Ebook => "ebooks",
        }
    }
    const fn column(self) -> &'static str {
        match self {
            Target::Course => "course_id",
            Target::Ebook => "ebook_id",
        }
    }
    const fn owner_table(self) -> &'static str {
        match self {
            Target::Course => "enrollments",
            Target::Ebook => "ebook_purchases",
        }
    }
}

pub(crate) const REVIEW_SELECT: &str = "SELECT r.id, u.full_name AS user_name, r.rating, r.comment, r.updated_at \
     FROM reviews r JOIN users u ON u.id = r.user_id";

/// Latest reviews (with comments first) and the caller's own review, for a detail page.
pub(crate) async fn for_item(
    db: &PgPool,
    target: Target,
    item_id: Uuid,
    user_id: Option<Uuid>,
) -> sqlx::Result<(Vec<Review>, Option<Review>)> {
    let col = target.column();
    let list_sql = format!(
        "{REVIEW_SELECT} WHERE r.{col} = $1 ORDER BY (r.comment IS NOT NULL) DESC, r.updated_at DESC LIMIT 10"
    );
    let mine_sql = format!("{REVIEW_SELECT} WHERE r.{col} = $1 AND r.user_id = $2");
    tokio::try_join!(
        sqlx::query_as::<_, Review>(&list_sql).bind(item_id).fetch_all(db),
        sqlx::query_as::<_, Review>(&mine_sql).bind(item_id).bind(user_id).fetch_optional(db),
    )
}

/// Rating 1–5; comment trimmed, blank → `None`, at most 2000 characters.
fn validate_review(rating: i16, comment: Option<&str>) -> AppResult<(i16, Option<String>)> {
    if !(1..=5).contains(&rating) {
        return Err(AppError::Validation("invalid_rating"));
    }
    let comment = comment.map(str::trim).filter(|c| !c.is_empty());
    if comment.is_some_and(|c| c.chars().count() > 2000) {
        return Err(AppError::Validation("invalid_comment"));
    }
    Ok((rating, comment.map(str::to_owned)))
}

/// The item's id if the user owns it: 404 unknown slug (or an unpublished one they don't own), 403 not owned.
async fn owned_item(db: &PgPool, target: Target, slug: &str, user_id: Uuid) -> AppResult<Uuid> {
    let sql = format!(
        "SELECT t.id, t.published, EXISTS (SELECT 1 FROM {owner} o WHERE o.{col} = t.id AND o.user_id = $2) \
         FROM {table} t WHERE t.slug = $1",
        owner = target.owner_table(),
        col = target.column(),
        table = target.table(),
    );
    let (id, published, owned): (Uuid, bool, bool) = sqlx::query_as(&sql)
        .bind(slug)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::NotFound)?;
    match (owned, published) {
        (true, _) => Ok(id),
        (false, true) => Err(AppError::Forbidden),
        (false, false) => Err(AppError::NotFound),
    }
}

#[derive(Deserialize)]
pub struct ReviewBody {
    rating: i16,
    comment: Option<String>,
}

async fn upsert(st: AppState, user_id: Uuid, target: Target, slug: String, body: ReviewBody) -> AppResult<Json<Review>> {
    let (rating, comment) = validate_review(body.rating, body.comment.as_deref())?;
    let item_id = owned_item(&st.db, target, &slug, user_id).await?;
    let col = target.column();
    let sql = format!(
        "WITH r AS (
             INSERT INTO reviews (user_id, {col}, rating, comment) VALUES ($1, $2, $3, $4)
             ON CONFLICT (user_id, {col}) WHERE {col} IS NOT NULL
             DO UPDATE SET rating = EXCLUDED.rating, comment = EXCLUDED.comment, updated_at = now()
             RETURNING *
         )
         SELECT r.id, u.full_name AS user_name, r.rating, r.comment, r.updated_at FROM r JOIN users u ON u.id = r.user_id"
    );
    let review = sqlx::query_as::<_, Review>(&sql)
        .bind(user_id)
        .bind(item_id)
        .bind(rating)
        .bind(comment)
        .fetch_one(&st.db)
        .await?;
    Ok(Json(review))
}

async fn remove(st: AppState, user_id: Uuid, target: Target, slug: String) -> AppResult<StatusCode> {
    let sql = format!(
        "DELETE FROM reviews r USING {table} t WHERE r.{col} = t.id AND t.slug = $1 AND r.user_id = $2",
        table = target.table(),
        col = target.column(),
    );
    sqlx::query(&sql).bind(slug).bind(user_id).execute(&st.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn put_course(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(slug): Path<String>,
    Json(body): Json<ReviewBody>,
) -> AppResult<Json<Review>> {
    upsert(st, current.user.id, Target::Course, slug, body).await
}

pub async fn put_ebook(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(slug): Path<String>,
    Json(body): Json<ReviewBody>,
) -> AppResult<Json<Review>> {
    upsert(st, current.user.id, Target::Ebook, slug, body).await
}

pub async fn delete_course(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(slug): Path<String>,
) -> AppResult<StatusCode> {
    remove(st, current.user.id, Target::Course, slug).await
}

pub async fn delete_ebook(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(slug): Path<String>,
) -> AppResult<StatusCode> {
    remove(st, current.user.id, Target::Ebook, slug).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code<T>(r: AppResult<T>) -> &'static str {
        match r {
            Err(AppError::Validation(c)) => c,
            Err(_) => "other",
            Ok(_) => "ok",
        }
    }

    #[test]
    fn rating_must_be_one_to_five() {
        assert_eq!(code(validate_review(0, None)), "invalid_rating");
        assert_eq!(code(validate_review(6, None)), "invalid_rating");
        assert_eq!(code(validate_review(1, None)), "ok");
        assert_eq!(code(validate_review(5, None)), "ok");
    }

    #[test]
    fn comment_is_trimmed_and_blank_is_none() {
        assert_eq!(validate_review(4, Some("  ល្អណាស់ ")).unwrap(), (4, Some("ល្អណាស់".into())));
        assert_eq!(validate_review(4, Some("   ")).unwrap(), (4, None));
    }

    #[test]
    fn comment_is_capped_at_2000_characters() {
        assert_eq!(code(validate_review(4, Some(&"ក".repeat(2000)))), "ok");
        assert_eq!(code(validate_review(4, Some(&"ក".repeat(2001)))), "invalid_comment");
    }
}
