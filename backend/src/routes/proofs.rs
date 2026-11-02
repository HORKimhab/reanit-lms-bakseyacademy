//! Payment screenshots: the buyer attaches one to their order; the bot posts it (with the order
//! details as caption) to the admins' Telegram chat, and admins see it in the review queue.

use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use chrono::Duration;
use uuid::Uuid;

use super::{
    media::{insert_telegram, signed_url, upload_form_to_telegram},
    orders::load_order,
};
use crate::{
    AppState,
    auth::CurrentUser,
    error::{AppError, AppResult},
    models::{OrderDetail, OrderSummary},
};

/// Screenshots are small; anything bigger is not a screenshot.
const MAX_PROOF_BYTES: u64 = 10 * 1024 * 1024;

/// `POST /api/orders/{id}/proof`, multipart `size` then `file` (PNG/JPEG/WebP ≤ 10 MB): attaches a
/// payment screenshot to the buyer's own unpaid order and posts it, captioned with the order details,
/// to the admins' Telegram chat. Replaces an earlier screenshot.
/// 404 not their order, 409 `invalid_status` once paid/cancelled, 422 `invalid_image` / `invalid_size`
/// / `file_too_large`, 503 `telegram_unavailable` / `telegram_failed`.
pub async fn upload(
    State(st): State<AppState>,
    current: CurrentUser,
    Path(id): Path<Uuid>,
    form: Multipart,
) -> AppResult<Json<OrderDetail>> {
    let tg = st.telegram.as_ref().ok_or(AppError::Unavailable("telegram_unavailable"))?;
    let order = load_order(&st, id, Some(current.user.id)).await?;
    if !matches!(order.order.status.as_str(), "pending_payment" | "awaiting_approval" | "rejected") {
        return Err(AppError::Conflict("invalid_status"));
    }
    let caption = proof_caption(&ProofContext {
        order_id: id,
        buyer: &current.user.full_name,
        phone: current.user.phone.as_deref(),
        total_cents: order.order.total_cents,
        method: order.order.payment_method_code.as_deref(),
        payment_ref: order.order.payment_ref.as_deref(),
    });
    let stored = upload_form_to_telegram(tg, form, validate_proof, tg.admin_chat_id(), Some(&caption)).await?;
    let media = insert_telegram(&st.db, &stored).await?;
    sqlx::query("UPDATE orders SET payment_proof_media_id = $2 WHERE id = $1 AND user_id = $3")
        .bind(id)
        .bind(media.id)
        .bind(current.user.id)
        .execute(&st.db)
        .await?;
    Ok(Json(load_order(&st, id, Some(current.user.id)).await?))
}

/// Fills `payment_proof_url` (signed, 1 h) when the order has a screenshot and Telegram is on.
pub(crate) fn attach_proof_url(st: &AppState, order: &mut OrderSummary) {
    if let (Some(media_id), Some(_)) = (order.payment_proof_media_id, &st.telegram) {
        order.payment_proof_url = Some(signed_url(st, media_id, Duration::hours(1), false).0);
    }
}

/// What the admins' Telegram caption says about the order.
pub struct ProofContext<'a> {
    pub order_id: uuid::Uuid,
    pub buyer: &'a str,
    pub phone: Option<&'a str>,
    pub total_cents: i32,
    pub method: Option<&'a str>,
    pub payment_ref: Option<&'a str>,
}

/// Telegram caption (≤ 1024 chars) for a payment screenshot.
fn proof_caption(ctx: &ProofContext) -> String {
    let short_id: String = ctx.order_id.to_string().chars().take(8).collect::<String>().to_uppercase();
    let dollars = f64::from(ctx.total_cents) / 100.0;
    let amount = if ctx.total_cents % 100 == 0 { format!("${}", ctx.total_cents / 100) } else { format!("${dollars:.2}") };
    let mut lines = vec![format!("🧾 ការបញ្ជាទិញ #{short_id}")];
    lines.push(match ctx.phone {
        Some(phone) => format!("👤 {} · {phone}", ctx.buyer),
        None => format!("👤 {}", ctx.buyer),
    });
    lines.push(match ctx.method {
        Some(method) => format!("💵 {amount} · {method}"),
        None => format!("💵 {amount}"),
    });
    if let Some(r) = ctx.payment_ref {
        lines.push(format!("🔖 {r}"));
    }
    lines.join("\n").chars().take(1024).collect()
}

/// PNG / JPEG / WebP up to 10 MB.
fn validate_proof(mime: &str, size: u64) -> AppResult<()> {
    if !matches!(mime, "image/png" | "image/jpeg" | "image/webp") {
        return Err(AppError::Validation("invalid_image"));
    }
    if size == 0 {
        return Err(AppError::Validation("invalid_size"));
    }
    if size > MAX_PROOF_BYTES {
        return Err(AppError::Validation("file_too_large"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(r: AppResult<()>) -> &'static str {
        match r {
            Ok(()) => "ok",
            Err(AppError::Validation(c)) => c,
            Err(_) => "other",
        }
    }

    #[test]
    fn caption_lists_order_buyer_amount_and_reference() {
        let ctx = ProofContext {
            order_id: "abcd1234-0000-0000-0000-000000000000".parse().unwrap(),
            buyer: "សុខ ដារ៉ា",
            phone: Some("+85512345678"),
            total_cents: 1550,
            method: Some("aba_payway"),
            payment_ref: Some("TX-991"),
        };
        assert_eq!(
            proof_caption(&ctx),
            "🧾 ការបញ្ជាទិញ #ABCD1234\n👤 សុខ ដារ៉ា · +85512345678\n💵 $15.50 · aba_payway\n🔖 TX-991"
        );
        let bare = ProofContext { phone: None, total_cents: 3000, method: None, payment_ref: None, ..ctx };
        assert_eq!(proof_caption(&bare), "🧾 ការបញ្ជាទិញ #ABCD1234\n👤 សុខ ដារ៉ា\n💵 $30");
    }

    #[test]
    fn caption_is_capped_for_telegram() {
        let long = "x".repeat(5000);
        let ctx = ProofContext {
            order_id: uuid::Uuid::nil(),
            buyer: &long,
            phone: None,
            total_cents: 100,
            method: None,
            payment_ref: Some(&long),
        };
        assert!(proof_caption(&ctx).chars().count() <= 1024);
    }

    #[test]
    fn only_reasonable_images() {
        assert_eq!(code(validate_proof("image/png", 200_000)), "ok");
        assert_eq!(code(validate_proof("image/jpeg", MAX_PROOF_BYTES)), "ok");
        assert_eq!(code(validate_proof("image/webp", 1)), "ok");
        assert_eq!(code(validate_proof("application/pdf", 1)), "invalid_image");
        assert_eq!(code(validate_proof("image/svg+xml", 1)), "invalid_image");
        assert_eq!(code(validate_proof("image/png", 0)), "invalid_size");
        assert_eq!(code(validate_proof("image/png", MAX_PROOF_BYTES + 1)), "file_too_large");
    }
}
