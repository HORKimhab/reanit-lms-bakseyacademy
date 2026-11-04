//! Discount codes: redemption rules shared by the cart preview and checkout, and admin CRUD.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{FromRow, PgConnection};
use uuid::Uuid;

use crate::{
    AppState,
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{AdminCoupon, AppliedCoupon},
    validate,
};

/// Upper-cased code, or `invalid_coupon` when it can't be one (3–32 of `A–Z 0–9 _ -`).
pub fn normalize_code(code: &str) -> AppResult<String> {
    let code = code.trim().to_ascii_uppercase();
    let valid = (3..=32).contains(&code.len())
        && code.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' || b == b'-');
    if !valid {
        return Err(AppError::Validation("invalid_coupon"));
    }
    Ok(code)
}

/// What a coupon takes off `subtotal`: never more than the subtotal; percentages round down.
pub fn discount(percent_off: Option<i32>, amount_off_cents: Option<i32>, subtotal: i64) -> i64 {
    let off = match (percent_off, amount_off_cents) {
        (Some(p), _) => subtotal * i64::from(p.clamp(0, 100)) / 100,
        (None, Some(a)) => i64::from(a.max(0)),
        (None, None) => 0,
    };
    off.min(subtotal)
}

#[derive(FromRow)]
struct CouponState {
    id: Uuid,
    code: String,
    percent_off: Option<i32>,
    amount_off_cents: Option<i32>,
    active: bool,
    expired: bool,
    used_up: bool,
}

/// A coupon the buyer may use now, with its discount on `subtotal`.
pub struct Redeemable {
    pub id: Uuid,
    pub applied: AppliedCoupon,
}

/// Looks up `code` and checks it can be redeemed: 422 `invalid_coupon` (unknown / inactive),
/// `coupon_expired`, `coupon_used_up`. With `lock` (inside the checkout transaction) the coupon row
/// stays locked until commit, so concurrent checkouts can't both take the last use.
pub async fn redeemable(conn: &mut PgConnection, code: &str, subtotal: i64, lock: bool) -> AppResult<Redeemable> {
    let code = normalize_code(code)?;
    if lock {
        // Lock first; the use count below then runs as a new statement that sees committed orders.
        sqlx::query("SELECT 1 FROM coupons WHERE code = $1 FOR UPDATE").bind(&code).execute(&mut *conn).await?;
    }
    let c = sqlx::query_as::<_, CouponState>(
        "SELECT c.id, c.code, c.percent_off, c.amount_off_cents, c.active, \
                COALESCE(c.expires_at <= now(), false) AS expired, \
                COALESCE(c.max_uses <= (SELECT COUNT(*) FROM orders o \
                    WHERE o.coupon_id = c.id AND o.status <> 'cancelled'), false) AS used_up \
         FROM coupons c WHERE c.code = $1",
    )
        .bind(&code)
        .fetch_optional(&mut *conn)
        .await?
        .filter(|c| c.active)
        .ok_or(AppError::Validation("invalid_coupon"))?;
    if c.expired {
        return Err(AppError::Validation("coupon_expired"));
    }
    if c.used_up {
        return Err(AppError::Validation("coupon_used_up"));
    }
    Ok(Redeemable {
        id: c.id,
        applied: AppliedCoupon {
            discount_cents: discount(c.percent_off, c.amount_off_cents, subtotal),
            code: c.code,
            percent_off: c.percent_off,
            amount_off_cents: c.amount_off_cents,
        },
    })
}

// ---- Admin ---------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CouponInput {
    code: String,
    percent_off: Option<i32>,
    amount_off_cents: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
    max_uses: Option<i32>,
    #[serde(default = "yes")]
    active: bool,
}

fn yes() -> bool {
    true
}

struct ValidCoupon {
    code: String,
    percent_off: Option<i32>,
    amount_off_cents: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
    max_uses: Option<i32>,
    active: bool,
}

impl CouponInput {
    fn validated(self) -> AppResult<ValidCoupon> {
        let valid_discount = match (self.percent_off, self.amount_off_cents) {
            (Some(p), None) => (1..=100).contains(&p),
            (None, Some(a)) => a > 0,
            _ => false,
        };
        if !valid_discount {
            return Err(AppError::Validation("invalid_discount"));
        }
        if self.max_uses.is_some_and(|m| m <= 0) {
            return Err(AppError::Validation("invalid_max_uses"));
        }
        Ok(ValidCoupon {
            code: normalize_code(&self.code)?,
            percent_off: self.percent_off,
            amount_off_cents: self.amount_off_cents,
            expires_at: self.expires_at,
            max_uses: self.max_uses,
            active: self.active,
        })
    }
}

const ADMIN_SELECT: &str = "SELECT c.id, c.code, c.percent_off, c.amount_off_cents, c.expires_at, c.max_uses, \
     c.active, c.created_at, \
     (SELECT COUNT(*) FROM orders o WHERE o.coupon_id = c.id AND o.status <> 'cancelled') AS uses, \
     (SELECT COALESCE(SUM(o.discount_cents), 0)::int8 FROM orders o WHERE o.coupon_id = c.id AND o.status = 'paid') \
        AS discount_given_cents \
     FROM coupons c";

async fn load(st: &AppState, id: Uuid) -> AppResult<AdminCoupon> {
    sqlx::query_as::<_, AdminCoupon>(&format!("{ADMIN_SELECT} WHERE c.id = $1"))
        .bind(id)
        .fetch_optional(&st.db)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list(State(st): State<AppState>, _: AdminUser) -> AppResult<Json<Vec<AdminCoupon>>> {
    let rows = sqlx::query_as::<_, AdminCoupon>(&format!("{ADMIN_SELECT} ORDER BY c.created_at DESC"))
        .fetch_all(&st.db)
        .await?;
    Ok(Json(rows))
}

pub async fn create(
    State(st): State<AppState>,
    _: AdminUser,
    Json(input): Json<CouponInput>,
) -> AppResult<(StatusCode, Json<AdminCoupon>)> {
    let c = input.validated()?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO coupons (code, percent_off, amount_off_cents, expires_at, max_uses, active) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(c.code)
    .bind(c.percent_off)
    .bind(c.amount_off_cents)
    .bind(c.expires_at)
    .bind(c.max_uses)
    .bind(c.active)
    .fetch_one(&st.db)
    .await
    .map_err(validate::unique_violation("code_taken"))?;
    Ok((StatusCode::CREATED, Json(load(&st, id).await?)))
}

pub async fn update(
    State(st): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<CouponInput>,
) -> AppResult<Json<AdminCoupon>> {
    let c = input.validated()?;
    let updated = sqlx::query(
        "UPDATE coupons SET code = $2, percent_off = $3, amount_off_cents = $4, expires_at = $5, \
         max_uses = $6, active = $7 WHERE id = $1",
    )
    .bind(id)
    .bind(c.code)
    .bind(c.percent_off)
    .bind(c.amount_off_cents)
    .bind(c.expires_at)
    .bind(c.max_uses)
    .bind(c.active)
    .execute(&st.db)
    .await
    .map_err(validate::unique_violation("code_taken"))?
    .rows_affected();
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(load(&st, id).await?))
}

/// Orders that used it keep their discount and `coupon_code` snapshot.
pub async fn delete(State(st): State<AppState>, _: AdminUser, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let deleted = sqlx::query("DELETE FROM coupons WHERE id = $1").bind(id).execute(&st.db).await?.rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discounts_round_down_and_never_exceed_the_subtotal() {
        assert_eq!(discount(Some(10), None, 1999), 199);
        assert_eq!(discount(Some(100), None, 1500), 1500);
        assert_eq!(discount(None, Some(500), 1500), 500);
        assert_eq!(discount(None, Some(5000), 1500), 1500);
        assert_eq!(discount(None, None, 1500), 0);
    }

    #[test]
    fn codes_are_upper_cased_and_checked() {
        assert_eq!(normalize_code(" new-year_2027 ").unwrap(), "NEW-YEAR_2027");
        for bad in ["", "ab", "has space", "ខ្មែរ", &"X".repeat(33)] {
            assert!(normalize_code(bad).is_err(), "{bad}");
        }
    }
}
