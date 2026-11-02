use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::{PgConnection, PgExecutor};
use uuid::Uuid;

use super::cart::load_cart;
use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::{OrderDetail, OrderItem, OrderSummary, PaymentMethod},
};

pub(crate) const ORDER_COLUMNS: &str = "o.id, o.status, o.total_cents, o.payment_method AS payment_method_code, \
     o.payment_ref, o.admin_note, o.created_at, o.submitted_at, o.paid_at";
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
    db: &sqlx::PgPool,
    id: Uuid,
    user_id: Option<Uuid>,
) -> AppResult<OrderDetail> {
    let order = sqlx::query_as::<_, OrderSummary>(&format!(
        "SELECT {ORDER_COLUMNS} FROM orders o WHERE o.id = $1 AND ($2::uuid IS NULL OR o.user_id = $2)"
    ))
    .bind(id)
    .bind(user_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)?;
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
    let cart = load_cart(&mut *tx, user_id).await?;
    if cart.items.is_empty() {
        return Err(AppError::Validation("cart_empty"));
    }
    let total =
        i32::try_from(cart.total_cents).map_err(|_| AppError::Validation("cart_too_large"))?;

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
        "INSERT INTO orders (user_id, status, payment_method, total_cents, paid_at) \
         VALUES ($1, $2, $3, $4, CASE WHEN $4 = 0 THEN now() END) RETURNING id",
    )
    .bind(user_id)
    .bind(if total == 0 {
        "paid"
    } else {
        "pending_payment"
    })
    .bind(method)
    .bind(total)
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
        Json(load_order(&st.db, order_id, Some(user_id)).await?),
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
    Ok(Json(load_order(&st.db, id, Some(current.user.id)).await?))
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
    let order = load_order(&st.db, id, Some(user_id)).await?;
    if updated == 0 {
        return Err(AppError::Conflict("invalid_status"));
    }
    Ok(Json(order))
}
