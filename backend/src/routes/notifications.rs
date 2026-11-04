//! In-app notices for the signed-in user (order approved / rejected).

use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{
    AppState,
    auth::CurrentUser,
    error::AppResult,
    models::{Notification, Notifications},
};

/// Records a notice for the owner of `order_id` (`kind`: `order_paid` / `order_rejected`).
pub(crate) async fn notify_order(db: impl PgExecutor<'_>, order_id: Uuid, kind: &str) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO notifications (user_id, kind, order_id, note) \
         SELECT user_id, $2, id, CASE WHEN $2 = 'order_rejected' THEN admin_note END FROM orders WHERE id = $1",
    )
    .bind(order_id)
    .bind(kind)
    .execute(db)
    .await?;
    Ok(())
}

/// Latest 20 notices, newest first, plus the unread count.
pub async fn list(State(st): State<AppState>, current: CurrentUser) -> AppResult<Json<Notifications>> {
    let (items, unread_count) = tokio::try_join!(
        sqlx::query_as::<_, Notification>(
            "SELECT id, kind, order_id, note, created_at, read_at FROM notifications \
             WHERE user_id = $1 ORDER BY created_at DESC LIMIT 20",
        )
        .bind(current.user.id)
        .fetch_all(&st.db),
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL")
            .bind(current.user.id)
            .fetch_one(&st.db),
    )?;
    Ok(Json(Notifications { items, unread_count }))
}

/// Marks every notice read.
pub async fn read_all(State(st): State<AppState>, current: CurrentUser) -> AppResult<StatusCode> {
    sqlx::query("UPDATE notifications SET read_at = now() WHERE user_id = $1 AND read_at IS NULL")
        .bind(current.user.id)
        .execute(&st.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
