//! The signed-in user's saved courses and ebooks.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use super::{courses, ebooks};
use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::{CourseCard, EbookCard, Wishlist, WishlistIds},
};

/// Saved items that are still published, most recently saved first.
pub async fn list(State(st): State<AppState>, current: CurrentUser) -> AppResult<Json<Wishlist>> {
    let course_sql = format!(
        "{} JOIN wishlist_items w ON w.course_id = c.id AND w.user_id = $1 WHERE c.published ORDER BY w.created_at DESC",
        courses::CARD_SELECT
    );
    let ebook_sql = format!(
        "{} JOIN wishlist_items w ON w.ebook_id = e.id AND w.user_id = $1 WHERE e.published ORDER BY w.created_at DESC",
        ebooks::CARD_SELECT
    );
    let (courses, ebooks) = tokio::try_join!(
        sqlx::query_as::<_, CourseCard>(&course_sql).bind(current.user.id).fetch_all(&st.db),
        sqlx::query_as::<_, EbookCard>(&ebook_sql).bind(current.user.id).fetch_all(&st.db),
    )?;
    Ok(Json(Wishlist { courses, ebooks }))
}

/// Just the saved ids, for filling in hearts across the site.
pub async fn ids(State(st): State<AppState>, current: CurrentUser) -> AppResult<Json<WishlistIds>> {
    let rows = sqlx::query_as::<_, (Option<Uuid>, Option<Uuid>)>(
        "SELECT course_id, ebook_id FROM wishlist_items WHERE user_id = $1",
    )
    .bind(current.user.id)
    .fetch_all(&st.db)
    .await?;
    Ok(Json(WishlistIds {
        course_ids: rows.iter().filter_map(|r| r.0).collect(),
        ebook_ids: rows.iter().filter_map(|r| r.1).collect(),
    }))
}

/// `kind` from the path: `courses` or `ebooks` → the wishlist column.
fn column(kind: &str) -> AppResult<&'static str> {
    match kind {
        "courses" => Ok("course_id"),
        "ebooks" => Ok("ebook_id"),
        _ => Err(AppError::NotFound),
    }
}

/// Saves a published course / ebook (idempotent). 404 for unknown or unpublished items.
pub async fn add(
    State(st): State<AppState>,
    current: CurrentUser,
    Path((kind, id)): Path<(String, Uuid)>,
) -> AppResult<StatusCode> {
    let col = column(&kind)?;
    let inserted = sqlx::query(&format!(
        "INSERT INTO wishlist_items (user_id, {col}) SELECT $1, id FROM {kind} WHERE id = $2 AND published \
         ON CONFLICT DO NOTHING"
    ))
    .bind(current.user.id)
    .bind(id)
    .execute(&st.db)
    .await?
    .rows_affected();
    if inserted == 0 {
        let exists: bool = sqlx::query_scalar(&format!("SELECT EXISTS (SELECT 1 FROM {kind} WHERE id = $1 AND published)"))
            .bind(id)
            .fetch_one(&st.db)
            .await?;
        if !exists {
            return Err(AppError::NotFound);
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove(
    State(st): State<AppState>,
    current: CurrentUser,
    Path((kind, id)): Path<(String, Uuid)>,
) -> AppResult<StatusCode> {
    let col = column(&kind)?;
    sqlx::query(&format!("DELETE FROM wishlist_items WHERE user_id = $1 AND {col} = $2"))
        .bind(current.user.id)
        .bind(id)
        .execute(&st.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
