use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::{PgConnection, PgExecutor};
use uuid::Uuid;

use super::{cart::load_cart, coupons, proofs};
use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::{OrderDetail, OrderItem, OrderSummary, OrderWithItems, PaymentMethod},
};

/// Orders that can still be paid (or resubmitted): a buyer may submit a reference or screenshot,
/// and their items count as "already ordered".
pub(crate) const PAYABLE_STATUSES: [&str; 3] = ["pending_payment", "awaiting_approval", "rejected"];

pub(crate) fn is_payable(status: &str) -> bool {
    PAYABLE_STATUSES.contains(&status)
}

/// The item (`$3` course / `$4` ebook) already sits in one of the user's (`$1`) payable orders
/// (`$2` = `PAYABLE_STATUSES`).
pub(crate) const IN_OPEN_ORDER: &str = "SELECT EXISTS (SELECT 1 FROM order_items oi JOIN orders o ON o.id = oi.order_id \
     WHERE o.user_id = $1 AND o.status = ANY($2) AND (oi.course_id = $3 OR oi.ebook_id = $4))";

pub(crate) const ORDER_COLUMNS: &str = "o.id, o.status, o.total_cents, o.discount_cents, o.coupon_code, o.payment_method AS payment_method_code, \
     o.payment_ref, o.admin_note, o.created_at, o.submitted_at, o.paid_at, o.payment_proof_media_id";
const METHOD_COLUMNS: &str = "code, name_km, name_en, account_name, account_number, qr_image_url";

pub async fn payment_methods(State(st): State<AppState>) -> AppResult<Json<Vec<PaymentMethod>>> {
    let rows = sqlx::query_as::<_, PaymentMethod>(&format!(
        "SELECT {METHOD_COLUMNS} FROM payment_methods WHERE active ORDER BY position, code"
    ))
    .fetch_all(&st.db)
    .await?;
    Ok(Json(rows))
}

/// Loads an order with items and payment method. `user_id = None` skips the owner check (admin).
pub(crate) async fn load_order(
    st: &AppState,
    id: Uuid,
    user_id: Option<Uuid>,
) -> AppResult<OrderDetail> {
    let db = &st.db;
    let mut order = sqlx::query_as::<_, OrderSummary>(&format!(
        "SELECT {ORDER_COLUMNS} FROM orders o WHERE o.id = $1 AND ($2::uuid IS NULL OR o.user_id = $2)"
    ))
    .bind(id)
    .bind(user_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)?;
    proofs::attach_proof_url(st, &mut order);
    let method_sql = format!("SELECT {METHOD_COLUMNS} FROM payment_methods WHERE code = $1");
    let (items, payment_method) = tokio::try_join!(
        sqlx::query_as::<_, OrderItem>(
            "SELECT kind, course_id, ebook_id, title, price_cents FROM order_items \
             WHERE order_id = $1 ORDER BY kind, title"
        )
        .bind(id)
        .fetch_all(db),
        sqlx::query_as::<_, PaymentMethod>(&method_sql)
            .bind(order.payment_method_code.as_deref())
            .fetch_optional(db),
    )?;
    Ok(OrderDetail {
        order,
        payment_method,
        items,
    })
}

/// Items for several orders at once, grouped by order id.
pub(crate) async fn items_by_order(
    db: &sqlx::PgPool,
    ids: &[Uuid],
) -> sqlx::Result<HashMap<Uuid, Vec<OrderItem>>> {
    let mut map: HashMap<Uuid, Vec<OrderItem>> = HashMap::new();
    for item in sqlx::query_as::<_, OrderItem>(
        "SELECT order_id, kind, course_id, ebook_id, title, price_cents FROM order_items \
         WHERE order_id = ANY($1) ORDER BY kind, title",
    )
    .bind(ids)
    .fetch_all(db)
    .await?
    {
        if let Some(order_id) = item.order_id {
            map.entry(order_id).or_default().push(item);
        }
    }
    Ok(map)
}

/// The signed-in user's orders, newest first (latest 100).
pub async fn my_orders(
    State(st): State<AppState>,
    current: CurrentUser,
) -> AppResult<Json<Vec<OrderWithItems>>> {
    let mut orders = sqlx::query_as::<_, OrderSummary>(&format!(
        "SELECT {ORDER_COLUMNS} FROM orders o WHERE o.user_id = $1 ORDER BY o.created_at DESC LIMIT 100"
    ))
    .bind(current.user.id)
    .fetch_all(&st.db)
    .await?;
    orders.iter_mut().for_each(|o| proofs::attach_proof_url(&st, o));
    let ids: Vec<Uuid> = orders.iter().map(|o| o.id).collect();
    let mut items = items_by_order(&st.db, &ids).await?;
    Ok(Json(
        orders
            .into_iter()
            .map(|order| OrderWithItems { items: items.remove(&order.id).unwrap_or_default(), order })
            .collect(),
    ))
}

/// Grants every course/ebook in a paid order. Idempotent.
pub(crate) async fn grant_order(conn: &mut PgConnection, order_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO enrollments (user_id, course_id, order_id) \
         SELECT o.user_id, oi.course_id, o.id FROM orders o JOIN order_items oi ON oi.order_id = o.id \
         WHERE o.id = $1 AND oi.course_id IS NOT NULL ON CONFLICT DO NOTHING",
    )
    .bind(order_id)
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "INSERT INTO ebook_purchases (user_id, ebook_id, order_id) \
         SELECT o.user_id, oi.ebook_id, o.id FROM orders o JOIN order_items oi ON oi.order_id = o.id \
         WHERE o.id = $1 AND oi.ebook_id IS NOT NULL ON CONFLICT DO NOTHING",
    )
    .bind(order_id)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct CheckoutBody {
    payment_method: Option<String>,
    /// Optional discount code (422 `invalid_coupon` / `coupon_expired` / `coupon_used_up`).
    coupon_code: Option<String>,
}

/// Turns the cart into an order (prices snapshotted) and empties the cart.
/// Items the user already owns are dropped. A free order is paid and granted immediately.
pub async fn checkout(
    State(st): State<AppState>,
    current: CurrentUser,
    Json(body): Json<CheckoutBody>,
) -> AppResult<(StatusCode, Json<OrderDetail>)> {
    let user_id = current.user.id;
    let mut tx = st.db.begin().await?;
    // Serialize concurrent checkouts by the same user.
    sqlx::query("SELECT 1 FROM users WHERE id = $1 FOR UPDATE")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    drop_owned_items(&mut *tx, user_id).await?;
    // Don't let the same item be ordered (and paid for) twice while an earlier order is open.
    let cart = load_cart(&mut *tx, user_id).await?;
    for item in &cart.items {
        let is_course = item.kind == "course";
        let ordered: bool = sqlx::query_scalar(IN_OPEN_ORDER)
            .bind(user_id)
            .bind(&PAYABLE_STATUSES[..])
            .bind(is_course.then_some(item.item_id))
            .bind((!is_course).then_some(item.item_id))
            .fetch_one(&mut *tx)
            .await?;
        if ordered {
            return Err(AppError::Conflict("already_ordered"));
        }
    }
    if cart.items.is_empty() {
        return Err(AppError::Validation("cart_empty"));
    }
    let coupon = match body.coupon_code.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        Some(code) => Some(coupons::redeemable(&mut tx, code, cart.subtotal_cents, true).await?),
        None => None,
    };
    let discount = coupon.as_ref().map_or(0, |c| c.applied.discount_cents);
    let total = i32::try_from(cart.subtotal_cents - discount)
        .map_err(|_| AppError::Validation("cart_too_large"))?;

    let method = if total == 0 {
        None
    } else {
        let code = body
            .payment_method
            .ok_or(AppError::Validation("invalid_payment_method"))?;
        let active: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM payment_methods WHERE code = $1 AND active)",
        )
        .bind(&code)
        .fetch_one(&mut *tx)
        .await?;
        if !active {
            return Err(AppError::Validation("invalid_payment_method"));
        }
        Some(code)
    };

    let order_id: Uuid = sqlx::query_scalar(
        "INSERT INTO orders (user_id, status, payment_method, total_cents, paid_at, coupon_id, coupon_code, discount_cents) \
         VALUES ($1, $2, $3, $4, CASE WHEN $4 = 0 THEN now() END, $5, $6, $7) RETURNING id",
    )
    .bind(user_id)
    .bind(if total == 0 {
        "paid"
    } else {
        "pending_payment"
    })
    .bind(method)
    .bind(total)
    .bind(coupon.as_ref().map(|c| c.id))
    .bind(coupon.as_ref().map(|c| c.applied.code.as_str()))
    .bind(discount as i32)
    .fetch_one(&mut *tx)
    .await?;
    for item in &cart.items {
        let is_course = item.kind == "course";
        sqlx::query(
            "INSERT INTO order_items (order_id, course_id, ebook_id, kind, title, price_cents) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(order_id)
        .bind(is_course.then_some(item.item_id))
        .bind((!is_course).then_some(item.item_id))
        .bind(&item.kind)
        .bind(&item.title)
        .bind(item.effective_price_cents())
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    if total == 0 {
        grant_order(&mut tx, order_id).await?;
    }
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(load_order(&st, order_id, Some(user_id)).await?),
    ))
}

async fn drop_owned_items(db: impl PgExecutor<'_>, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "DELETE FROM cart_items ci WHERE ci.user_id = $1 AND ( \
           EXISTS (SELECT 1 FROM enrollments en WHERE en.user_id = $1 AND en.course_id = ci.course_id) OR \
           EXISTS (SELECT 1 FROM ebook_purchases ep WHERE ep.user_id = $1 AND ep.ebook_id = ci.ebook_id))",
    )
    .bind(user_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn detail(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderDetail>> {
    Ok(Json(load_order(&st, id, Some(current.user.id)).await?))
}

#[derive(Deserialize)]
pub struct SubmitBody {
    payment_ref: String,
}

/// The user says they paid: records the transaction reference and queues the order for an admin.
/// Allowed from `pending_payment`, and from `rejected` to resubmit.
pub async fn submit(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SubmitBody>,
) -> AppResult<Json<OrderDetail>> {
    let payment_ref = body.payment_ref.trim();
    if !(3..=100).contains(&payment_ref.chars().count()) {
        return Err(AppError::Validation("invalid_payment_ref"));
    }
    transition(
        &st,
        id,
        current.user.id,
        "UPDATE orders SET status = 'awaiting_approval', payment_ref = $3, submitted_at = now(), \
         admin_note = NULL WHERE id = $1 AND user_id = $2 AND status IN ('pending_payment', 'rejected')",
        Some(payment_ref),
    )
    .await
}

/// Cancels an order the user hasn't paid for yet.
pub async fn cancel(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderDetail>> {
    transition(
        &st,
        id,
        current.user.id,
        "UPDATE orders SET status = 'cancelled' \
         WHERE id = $1 AND user_id = $2 AND status = 'pending_payment' AND $3::text IS NULL",
        None,
    )
    .await
}

/// Runs a guarded status update: 404 if the order isn't the user's, 409 `invalid_status` if the
/// guard didn't match.
async fn transition(
    st: &AppState,
    id: Uuid,
    user_id: Uuid,
    sql: &str,
    arg: Option<&str>,
) -> AppResult<Json<OrderDetail>> {
    let updated = sqlx::query(sql)
        .bind(id)
        .bind(user_id)
        .bind(arg)
        .execute(&st.db)
        .await?
        .rows_affected();
    let order = load_order(st, id, Some(user_id)).await?;
    if updated == 0 {
        return Err(AppError::Conflict("invalid_status"));
    }
    Ok(Json(order))
}
