use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    orders::{ORDER_COLUMNS, grant_order, items_by_order, load_order},
    proofs,
};
use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{AdminOrder, AdminOrderRow, ListParams, OrderDetail, Page},
};

const STATUSES: [&str; 5] = [
    "pending_payment",
    "awaiting_approval",
    "paid",
    "rejected",
    "cancelled",
];

const ORDER_FILTER: &str = r#"
FROM orders o JOIN users u ON u.id = o.user_id
WHERE ($1::text IS NULL OR o.status = $1)
  AND ($2::text IS NULL OR u.full_name ILIKE $2 OR u.phone ILIKE $2 OR o.payment_ref ILIKE $2
       OR o.id::text ILIKE $2)
"#;

#[derive(Deserialize)]
pub struct StatusParam {
    status: Option<String>,
}

/// Review queue. `?status=` filters (default: all), `?q=` searches name, phone, reference or order id.
/// Oldest first for `awaiting_approval` so the queue is worked in order; newest first otherwise.
pub async fn list_orders(
    State(st): State<AppState>,
    _admin: AdminUser,
    Query(p): Query<ListParams>,
    Query(sp): Query<StatusParam>,
) -> AppResult<Json<Page<AdminOrder>>> {
    let status = sp.status.filter(|s| !s.is_empty());
    if status.as_deref().is_some_and(|s| !STATUSES.contains(&s)) {
        return Err(AppError::Validation("invalid_status"));
    }
    let pattern = p.search_pattern();
    let order = if status.as_deref() == Some("awaiting_approval") {
        "ASC"
    } else {
        "DESC"
    };

    let list_sql = format!(
        "SELECT {ORDER_COLUMNS}, u.full_name AS user_full_name, u.phone AS user_phone {ORDER_FILTER} \
         ORDER BY COALESCE(o.submitted_at, o.created_at) {order} LIMIT $3 OFFSET $4"
    );
    let count_sql = format!("SELECT COUNT(*) {ORDER_FILTER}");
    let (rows, total) = tokio::try_join!(
        sqlx::query_as::<_, AdminOrderRow>(&list_sql)
            .bind(status.as_deref())
            .bind(pattern.as_deref())
            .bind(p.per_page())
            .bind(p.offset())
            .fetch_all(&st.db),
        sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(status.as_deref())
            .bind(pattern.as_deref())
            .fetch_one(&st.db),
    )?;

    let ids: Vec<Uuid> = rows.iter().map(|r| r.order.id).collect();
    let mut items_by_order = items_by_order(&st.db, &ids).await?;

    let items = rows
        .into_iter()
        .map(|mut r| AdminOrder {
            items: items_by_order.remove(&r.order.id).unwrap_or_default(),
            order: {
                proofs::attach_proof_url(&st, &mut r.order);
                r.order
            },
            user_full_name: r.user_full_name,
            user_phone: r.user_phone,
        })
        .collect();
    Ok(Json(Page {
        items,
        total,
        page: p.page(),
        per_page: p.per_page(),
    }))
}

/// Marks the order paid and grants its items. Approving an already-paid order is a no-op.
/// Admins may approve straight from `pending_payment` or `rejected` (e.g. payment confirmed by phone).
pub async fn approve(
    State(st): State<AppState>,
    AdminUser(admin): AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderDetail>> {
    let mut tx = st.db.begin().await?;
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM orders WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    match status.as_deref() {
        None => return Err(AppError::NotFound),
        Some("paid") => {}
        Some("pending_payment" | "awaiting_approval" | "rejected") => {
            sqlx::query(
                "UPDATE orders SET status = 'paid', paid_at = now(), reviewed_by = $2, reviewed_at = now(), \
                 admin_note = NULL WHERE id = $1",
            )
            .bind(id)
            .bind(admin.id)
            .execute(&mut *tx)
            .await?;
            grant_order(&mut tx, id).await?;
        }
        Some(_) => return Err(AppError::Conflict("invalid_status")),
    }
    tx.commit().await?;
    Ok(Json(load_order(&st, id, None).await?))
}

#[derive(Deserialize)]
pub struct RejectBody {
    note: String,
}

/// Rejects a submitted payment with a note the user sees; they can resubmit a reference.
pub async fn reject(
    State(st): State<AppState>,
    AdminUser(admin): AdminUser,
    Path(id): Path<Uuid>,
    Json(body): Json<RejectBody>,
) -> AppResult<Json<OrderDetail>> {
    let note = body.note.trim();
    if note.is_empty() || note.chars().count() > 500 {
        return Err(AppError::Validation("invalid_note"));
    }
    let updated = sqlx::query(
        "UPDATE orders SET status = 'rejected', admin_note = $3, reviewed_by = $2, reviewed_at = now() \
         WHERE id = $1 AND status = 'awaiting_approval'",
    )
    .bind(id)
    .bind(admin.id)
    .bind(note)
    .execute(&st.db)
    .await?
    .rows_affected();
    let order = load_order(&st, id, None).await?;
    if updated == 0 {
        return Err(AppError::Conflict("invalid_status"));
    }
    Ok(Json(order))
}
