//! Admin dashboard numbers: the approval queue, revenue over time and best sellers.

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{AdminStats, DailyRevenue, OrderCounts, TopItem},
};

#[derive(Deserialize)]
pub struct StatsParams {
    /// Period for the daily chart, totals and best sellers: 7–365 days (default 30), ending today
    /// (Phnom Penh time).
    days: Option<i32>,
}

/// Best sellers in the period (`$1` = kind, `$2` = days), by paid units then revenue.
const TOP_ITEMS: &str = r#"
SELECT COALESCE(c.id, e.id) AS id, COALESCE(c.slug, e.slug) AS slug, MAX(oi.title) AS title,
       COUNT(*) AS sales, SUM(oi.price_cents)::int8 AS revenue_cents
FROM order_items oi
JOIN orders o ON o.id = oi.order_id
LEFT JOIN courses c ON c.id = oi.course_id
LEFT JOIN ebooks e ON e.id = oi.ebook_id
WHERE o.status = 'paid' AND oi.kind = $1 AND COALESCE(c.id, e.id) IS NOT NULL
  AND o.paid_at >= (date_trunc('day', now() AT TIME ZONE 'Asia/Phnom_Penh') - make_interval(days => $2 - 1))
                   AT TIME ZONE 'Asia/Phnom_Penh'
GROUP BY 1, 2
ORDER BY sales DESC, revenue_cents DESC
LIMIT 5
"#;

pub async fn stats(
    State(st): State<AppState>,
    _admin: AdminUser,
    Query(p): Query<StatsParams>,
) -> AppResult<Json<AdminStats>> {
    let days = p.days.unwrap_or(30);
    if !(7..=365).contains(&days) {
        return Err(AppError::Validation("invalid_days"));
    }
    let (counts, daily, top_courses, top_ebooks) = tokio::try_join!(
        sqlx::query_as::<_, OrderCounts>(
            "SELECT
                COUNT(*) FILTER (WHERE status = 'awaiting_approval') AS awaiting_approval,
                COUNT(*) FILTER (WHERE status = 'pending_payment') AS pending_payment,
                COALESCE(SUM(total_cents) FILTER (WHERE status = 'paid'), 0)::int8 AS revenue_cents_all_time,
                (SELECT COUNT(*) FROM users) AS users,
                (SELECT COUNT(*) FROM users WHERE created_at >= now() - make_interval(days => $1)) AS new_users
             FROM orders",
        )
        .bind(days)
        .fetch_one(&st.db),
        // One row per day in the period, zero-filled, in Phnom Penh time.
        sqlx::query_as::<_, DailyRevenue>(
            "WITH days AS (
                SELECT generate_series(
                    (now() AT TIME ZONE 'Asia/Phnom_Penh')::date - ($1 - 1),
                    (now() AT TIME ZONE 'Asia/Phnom_Penh')::date,
                    interval '1 day')::date AS day
             )
             SELECT d.day, COALESCE(SUM(o.total_cents), 0)::int8 AS revenue_cents, COUNT(o.id) AS orders
             FROM days d
             LEFT JOIN orders o ON o.status = 'paid' AND o.total_cents > 0
                AND (o.paid_at AT TIME ZONE 'Asia/Phnom_Penh')::date = d.day
             GROUP BY d.day ORDER BY d.day",
        )
        .bind(days)
        .fetch_all(&st.db),
        sqlx::query_as::<_, TopItem>(TOP_ITEMS).bind("course").bind(days).fetch_all(&st.db),
        sqlx::query_as::<_, TopItem>(TOP_ITEMS).bind("ebook").bind(days).fetch_all(&st.db),
    )?;
    Ok(Json(AdminStats {
        days,
        revenue_cents: daily.iter().map(|d| d.revenue_cents).sum(),
        paid_orders: daily.iter().map(|d| d.orders).sum(),
        counts,
        daily,
        top_courses,
        top_ebooks,
    }))
}
