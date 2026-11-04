//! Hourly housekeeping: deletes expired sessions and cancels long-abandoned unpaid orders (which
//! would otherwise keep their items "already ordered" and hold coupon uses).

use std::time::Duration;

use sqlx::PgPool;

/// What one pass removed.
#[derive(Debug, PartialEq, Eq)]
pub struct Swept {
    pub sessions: u64,
    pub orders: u64,
}

/// One pass. Only `pending_payment` orders older than `abandon_days` are cancelled: once a buyer has
/// submitted a reference or screenshot (`awaiting_approval`) or been rejected, an admin decides.
pub async fn run_once(db: &PgPool, abandon_days: i32) -> sqlx::Result<Swept> {
    let sessions = sqlx::query("DELETE FROM sessions WHERE expires_at <= now()").execute(db).await?.rows_affected();
    let orders = sqlx::query(
        "UPDATE orders SET status = 'cancelled' \
         WHERE status = 'pending_payment' AND created_at < now() - make_interval(days => $1)",
    )
    .bind(abandon_days)
    .execute(db)
    .await?
    .rows_affected();
    Ok(Swept { sessions, orders })
}

/// Runs [`run_once`] now and then every hour, logging what it removed.
pub fn spawn(db: PgPool, abandon_days: i32) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(60 * 60));
        loop {
            tick.tick().await;
            match run_once(&db, abandon_days).await {
                Ok(Swept { sessions: 0, orders: 0 }) => {}
                Ok(s) => tracing::info!(sessions = s.sessions, orders = s.orders, "cleanup: removed expired sessions, cancelled abandoned orders"),
                Err(e) => tracing::warn!(error = %e, "cleanup failed"),
            }
        }
    });
}
