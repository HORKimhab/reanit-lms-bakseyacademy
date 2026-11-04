//! Automatic payment confirmation. Buyers get a dynamic KHQR (Bakong) or a signed PayWay checkout
//! for their order; we then ask the provider whether that exact attempt was paid — when the buyer's
//! page polls, when PayWay pushes back, before a cancel, and from a background poller. Nothing the
//! client or a pushback claims is trusted: only the provider's answer, and only when amount,
//! currency and (Bakong) recipient match the attempt.

use std::time::Duration;

use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::FromRow;
use uuid::Uuid;

use super::orders::{PaidVia, Settled, is_payable, load_order, settle};
use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::{KhqrPayment, OrderDetail, PaywayCheckout},
    payments::{BAKONG_METHOD, Lookup, PAYWAY_METHOD, khqr, payway},
};

/// A KHQR stays scannable this long (it carries its own expiry).
const KHQR_TTL: chrono::Duration = chrono::Duration::minutes(15);
/// PayWay checkout `lifetime`.
const PAYWAY_LIFETIME_MINUTES: u32 = 30;
/// Keep looking an attempt up this long after it expires (a payment made at the last second).
const GRACE: chrono::Duration = chrono::Duration::minutes(10);

#[derive(FromRow)]
struct PayableOrder {
    status: String,
    total_cents: i32,
    payment_method: Option<String>,
}

/// The caller's order, if it can still be paid with `method` through a configured provider:
/// 404 not theirs, 409 `invalid_status`, 422 `invalid_payment_method`, 503 `payment_unavailable`.
async fn payable(st: &AppState, user_id: Uuid, order_id: Uuid, method: &str) -> AppResult<i32> {
    let o = sqlx::query_as::<_, PayableOrder>(
        "SELECT status, total_cents, payment_method FROM orders WHERE id = $1 AND user_id = $2",
    )
    .bind(order_id)
    .bind(user_id)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    if !is_payable(&o.status) || o.total_cents <= 0 {
        return Err(AppError::Conflict("invalid_status"));
    }
    if o.payment_method.as_deref() != Some(method) {
        return Err(AppError::Validation("invalid_payment_method"));
    }
    if !st.payments.automatic(method) {
        return Err(AppError::Unavailable("payment_unavailable"));
    }
    Ok(o.total_cents)
}

/// `POST /api/orders/{id}/khqr`: the order's current dynamic KHQR (a new one when the last expires).
pub async fn khqr(State(st): State<AppState>, current: CurrentUser, Path(id): Path<Uuid>) -> AppResult<Json<KhqrPayment>> {
    let total = payable(&st, current.user.id, id, BAKONG_METHOD).await?;
    let bakong = st.payments.bakong.as_ref().ok_or(AppError::Unavailable("payment_unavailable"))?;
    let live: Option<(String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT qr, expires_at FROM payment_intents
         WHERE order_id = $1 AND provider = 'bakong' AND status = 'open' AND amount_cents = $2
           AND expires_at > now() + interval '1 minute'
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(id)
    .bind(total)
    .fetch_optional(&st.db)
    .await?;
    if let Some((qr, expires_at)) = live {
        return Ok(Json(KhqrPayment { qr, amount_cents: total, expires_at }));
    }
    let now = Utc::now();
    let expires_at = now + KHQR_TTL;
    let bill = id.simple().to_string()[..8].to_ascii_uppercase();
    let qr = khqr::build(
        &bakong.merchant,
        &khqr::Payment {
            amount_cents: i64::from(total),
            bill_number: &bill,
            created_ms: now.timestamp_millis(),
            expires_ms: expires_at.timestamp_millis(),
        },
    );
    sqlx::query(
        "INSERT INTO payment_intents (order_id, provider, provider_ref, amount_cents, qr, expires_at)
         VALUES ($1, 'bakong', $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(khqr::md5_hex(&qr))
    .bind(total)
    .bind(&qr)
    .bind(expires_at)
    .execute(&st.db)
    .await?;
    Ok(Json(KhqrPayment { qr, amount_cents: total, expires_at }))
}

/// `POST /api/orders/{id}/payway`: a fresh signed checkout (every attempt has its own `tran_id`).
pub async fn payway(State(st): State<AppState>, current: CurrentUser, Path(id): Path<Uuid>) -> AppResult<Json<PaywayCheckout>> {
    let total = payable(&st, current.user.id, id, PAYWAY_METHOD).await?;
    let pw = st.payments.payway.as_ref().ok_or(AppError::Unavailable("payment_unavailable"))?;
    let items: Vec<(String, i64)> = sqlx::query_as::<_, (String, i32)>(
        "SELECT title, price_cents FROM order_items WHERE order_id = $1 ORDER BY kind, title",
    )
    .bind(id)
    .fetch_all(&st.db)
    .await?
    .into_iter()
    .map(|(title, price)| (title, i64::from(price)))
    .collect();
    let mut random = [0u8; 10];
    OsRng.fill_bytes(&mut random);
    let tran_id: String = random.iter().map(|b| format!("{b:02x}")).collect();
    let now = Utc::now();
    let firstname: String = current.user.full_name.chars().take(100).collect();
    let fields = payway::checkout_fields(
        &pw.config,
        &payway::Checkout {
            req_time: &now.format("%Y%m%d%H%M%S").to_string(),
            tran_id: &tran_id,
            amount_cents: i64::from(total),
            items: &items,
            firstname: &firstname,
            order_id: &id.to_string(),
            site_url: &pw.site_url,
            lifetime_minutes: PAYWAY_LIFETIME_MINUTES,
        },
    );
    sqlx::query(
        "INSERT INTO payment_intents (order_id, provider, provider_ref, amount_cents, expires_at)
         VALUES ($1, 'payway', $2, $3, $4)",
    )
    .bind(id)
    .bind(&tran_id)
    .bind(total)
    .bind(now + chrono::Duration::minutes(i64::from(PAYWAY_LIFETIME_MINUTES)))
    .execute(&st.db)
    .await?;
    Ok(Json(PaywayCheckout {
        action_url: format!("{}/api/payment-gateway/v1/payments/purchase", pw.config.api_url),
        fields,
    }))
}

/// `POST /api/orders/{id}/check-payment`: asks the provider about the order's open attempts.
pub async fn check_payment(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderDetail>> {
    load_order(&st, id, Some(current.user.id)).await?;
    check_order(&st, id).await?;
    Ok(Json(load_order(&st, id, Some(current.user.id)).await?))
}

/// `POST /api/payments/payway/pushback` (PayWay → storefront proxy → here). The body only says
/// which attempt to look at; the result comes from PayWay's check-transaction API.
pub async fn payway_pushback(State(st): State<AppState>, body: Bytes) -> AppResult<Json<Value>> {
    let tran_id = pushback_tran_id(&body).ok_or(AppError::NotFound)?;
    let intent = sqlx::query_as::<_, Intent>(&format!(
        "{INTENT_SELECT} WHERE provider = 'payway' AND provider_ref = $1"
    ))
    .bind(&tran_id)
    .fetch_optional(&st.db)
    .await?
    .ok_or(AppError::NotFound)?;
    if intent.status == "open" {
        check_intent(&st, &intent).await?;
    }
    Ok(Json(json!({ "status": "ok" })))
}

/// `tran_id` from a JSON or form-encoded pushback body.
fn pushback_tran_id(body: &[u8]) -> Option<String> {
    let id = match serde_json::from_slice::<Value>(body) {
        Ok(v) => v["tran_id"].as_str().map(str::to_owned),
        Err(_) => std::str::from_utf8(body)
            .ok()?
            .split('&')
            .find_map(|pair| pair.strip_prefix("tran_id="))
            .map(str::to_owned),
    }?;
    (!id.is_empty() && id.len() <= 20 && id.bytes().all(|b| b.is_ascii_alphanumeric())).then_some(id)
}

#[derive(FromRow)]
struct Intent {
    id: Uuid,
    order_id: Uuid,
    provider: String,
    provider_ref: String,
    amount_cents: i32,
    currency: String,
    status: String,
    expires_at: DateTime<Utc>,
}

const INTENT_SELECT: &str =
    "SELECT id, order_id, provider, provider_ref, amount_cents, currency, status, expires_at FROM payment_intents";

/// Checks every open attempt of an order (provider errors are logged, not returned).
pub(crate) async fn check_order(st: &AppState, order_id: Uuid) -> AppResult<()> {
    let intents = sqlx::query_as::<_, Intent>(&format!(
        "{INTENT_SELECT} WHERE order_id = $1 AND status = 'open' ORDER BY created_at"
    ))
    .bind(order_id)
    .fetch_all(&st.db)
    .await?;
    for intent in &intents {
        check_intent(st, intent).await?;
    }
    Ok(())
}

/// Asks the provider about one attempt and applies the answer.
async fn check_intent(st: &AppState, intent: &Intent) -> AppResult<()> {
    let provider: &'static str = match intent.provider.as_str() {
        "bakong" => "bakong",
        "payway" => "payway",
        _ => return Ok(()),
    };
    let Some(gateway) = st.payments.gateway(provider) else { return Ok(()) };
    sqlx::query("UPDATE payment_intents SET checked_at = now() WHERE id = $1").bind(intent.id).execute(&st.db).await?;
    let lookup = match gateway.lookup(&intent.provider_ref).await {
        Ok(lookup) => lookup,
        Err(e) => {
            tracing::warn!(error = %e, provider, intent = %intent.id, "payment lookup failed");
            return Ok(());
        }
    };
    let close = |status: &'static str| {
        sqlx::query("UPDATE payment_intents SET status = $2 WHERE id = $1 AND status = 'open'")
            .bind(intent.id)
            .bind(status)
            .execute(&st.db)
    };
    match lookup {
        Lookup::Pending => {
            if intent.expires_at + GRACE < Utc::now() {
                close("expired").await?;
            }
        }
        Lookup::Failed => {
            close("failed").await?;
        }
        Lookup::Paid { amount_cents, currency, to_account, provider_txn } => {
            let expected_to = st.payments.bakong.as_ref().map(|b| b.merchant.account_id.as_str());
            let recipient_ok = provider != "bakong" || to_account.as_deref() == expected_to;
            if amount_cents != i64::from(intent.amount_cents) || currency != intent.currency || !recipient_ok {
                tracing::error!(
                    provider, intent = %intent.id, order = %intent.order_id, amount_cents, currency, to_account,
                    "payment does not match the attempt; not granting"
                );
                close("failed").await?;
                return Ok(());
            }
            let mut tx = st.db.begin().await?;
            let open: Option<String> = sqlx::query_scalar("SELECT status FROM payment_intents WHERE id = $1 FOR UPDATE")
                .bind(intent.id)
                .fetch_optional(&mut *tx)
                .await?;
            if open.as_deref() != Some("open") {
                return Ok(());
            }
            match settle(&mut tx, intent.order_id, PaidVia::Provider(provider), Some(&provider_txn)).await? {
                Settled::Paid | Settled::AlreadyPaid => {}
                Settled::NotPayable | Settled::NotFound => tracing::error!(
                    provider, order = %intent.order_id, provider_txn,
                    "payment received for an order that can no longer be paid (cancelled?): refund or grant by hand"
                ),
            }
            sqlx::query("UPDATE payment_intents SET status = 'paid', provider_txn = $2, paid_at = now() WHERE id = $1")
                .bind(intent.id)
                .bind(&provider_txn)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
        }
    }
    Ok(())
}

/// One pass over every open attempt still worth asking about.
pub async fn poll_once(st: &AppState) -> AppResult<()> {
    let intents = sqlx::query_as::<_, Intent>(&format!(
        "{INTENT_SELECT} WHERE status = 'open' AND created_at > now() - interval '2 days' ORDER BY created_at"
    ))
    .fetch_all(&st.db)
    .await?;
    for intent in &intents {
        check_intent(st, intent).await?;
    }
    Ok(())
}

/// Confirms payments even when the buyer closed the page: polls every 20 s while a provider is on.
pub fn spawn_poller(st: AppState) {
    if st.payments.bakong.is_none() && st.payments.payway.is_none() {
        return;
    }
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(20));
        loop {
            tick.tick().await;
            if let Err(e) = poll_once(&st).await {
                tracing::warn!(error = %e, "payment poll failed");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushback_tran_id_from_json_or_form() {
        assert_eq!(pushback_tran_id(br#"{"tran_id":"abc123","status":"00"}"#).as_deref(), Some("abc123"));
        assert_eq!(pushback_tran_id(b"status=00&tran_id=abc123&apv=1").as_deref(), Some("abc123"));
        assert_eq!(pushback_tran_id(br#"{"tran_id":"' OR 1=1"}"#), None);
        assert_eq!(pushback_tran_id(b"nothing"), None);
    }
}
