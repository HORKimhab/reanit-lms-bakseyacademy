//! Automatic payment confirmation: Bakong KHQR and ABA PayWay. Pure payload code lives in the
//! provider modules; the network lookup sits behind [`Gateway`] so tests can swap in a fake.

pub mod bakong;
pub mod khqr;
pub mod payway;

use std::sync::Arc;

use futures_util::future::BoxFuture;

/// What a provider says about one payment (by its reference: KHQR md5 / PayWay tran_id).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lookup {
    /// Money received. The caller checks amount, currency and recipient before granting.
    Paid { amount_cents: i64, currency: String, to_account: Option<String>, provider_txn: String },
    /// Not paid (yet), or unknown to the provider.
    Pending,
    /// Declined, cancelled or refunded.
    Failed,
}

/// Looks a payment up at the provider.
pub trait Gateway: Send + Sync {
    fn lookup<'a>(&'a self, provider_ref: &'a str) -> BoxFuture<'a, anyhow::Result<Lookup>>;
}

/// Dollars as the providers report them (`15.5`) → cents.
pub(crate) fn cents(amount: f64) -> i64 {
    (amount * 100.0).round() as i64
}

#[derive(Clone)]
pub struct Bakong {
    pub merchant: khqr::Merchant,
    pub gateway: Arc<dyn Gateway>,
}

#[derive(Clone)]
pub struct Payway {
    pub config: payway::Config,
    pub gateway: Arc<dyn Gateway>,
    /// Public storefront origin (`https://…`): return / pushback URLs point here.
    pub site_url: String,
}

/// Configured providers; `None` = that method stays manual (static QR + admin approval).
#[derive(Clone, Default)]
pub struct Payments {
    pub bakong: Option<Bakong>,
    pub payway: Option<Payway>,
}

/// `payment_methods.code` values the providers take over.
pub const BAKONG_METHOD: &str = "bakong_khqr";
pub const PAYWAY_METHOD: &str = "aba_payway";

impl Payments {
    /// Payment for this method is confirmed automatically.
    pub fn automatic(&self, method: &str) -> bool {
        match method {
            BAKONG_METHOD => self.bakong.is_some(),
            PAYWAY_METHOD => self.payway.is_some(),
            _ => false,
        }
    }

    pub fn gateway(&self, provider: &str) -> Option<&Arc<dyn Gateway>> {
        match provider {
            "bakong" => self.bakong.as_ref().map(|b| &b.gateway),
            "payway" => self.payway.as_ref().map(|p| &p.gateway),
            _ => None,
        }
    }
}
