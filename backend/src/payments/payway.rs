//! ABA PayWay checkout: signed purchase form fields (HMAC-SHA512, Base64) and the
//! `check-transaction-2` lookup. Pushbacks are never trusted: they only trigger a lookup.

use std::time::Duration;

use anyhow::Context;
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use futures_util::future::BoxFuture;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha512;

use super::{Gateway, Lookup, cents};

#[derive(Clone, Debug)]
pub struct Config {
    pub merchant_id: String,
    pub api_key: String,
    /// `https://checkout-sandbox.payway.com.kh` or the production host.
    pub api_url: String,
}

/// One purchase on the hosted checkout.
pub struct Checkout<'a> {
    /// UTC `YYYYMMDDHHmmss`.
    pub req_time: &'a str,
    /// Unique per attempt, at most 20 characters.
    pub tran_id: &'a str,
    pub amount_cents: i64,
    /// `(title, price in cents)` lines, descriptive only.
    pub items: &'a [(String, i64)],
    pub firstname: &'a str,
    pub order_id: &'a str,
    /// Storefront origin, no trailing slash.
    pub site_url: &'a str,
    pub lifetime_minutes: u32,
}

/// Base64(HMAC-SHA512(api_key, concatenated parts)).
pub fn sign(api_key: &str, parts: &[&str]) -> String {
    let mut mac = Hmac::<Sha512>::new_from_slice(api_key.as_bytes()).expect("HMAC takes any key length");
    for part in parts {
        mac.update(part.as_bytes());
    }
    B64.encode(mac.finalize().into_bytes())
}

fn dollars(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

/// Form fields to POST (multipart) to `{api_url}/api/payment-gateway/v1/payments/purchase`,
/// including `hash`; empty fields are left out (they still count, as "", in the hash).
pub fn checkout_fields(config: &Config, c: &Checkout) -> Vec<(&'static str, String)> {
    #[derive(serde::Serialize)]
    struct Item<'a> {
        name: &'a str,
        quantity: u32,
        price: f64,
    }
    let items: Vec<Item> = c.items.iter().map(|(name, price)| Item { name, quantity: 1, price: *price as f64 / 100.0 }).collect();
    let order_url = format!("{}/orders/{}", c.site_url, c.order_id);
    // Hash order from the Purchase API documentation; every field takes part, empty or not.
    let fields: [(&'static str, String); 24] = [
        ("req_time", c.req_time.to_owned()),
        ("merchant_id", config.merchant_id.clone()),
        ("tran_id", c.tran_id.to_owned()),
        ("amount", dollars(c.amount_cents)),
        ("items", B64.encode(serde_json::to_string(&items).expect("items serialize"))),
        ("shipping", String::new()),
        ("firstname", c.firstname.to_owned()),
        ("lastname", String::new()),
        ("email", String::new()),
        ("phone", String::new()),
        ("type", "purchase".into()),
        ("payment_option", String::new()),
        ("return_url", B64.encode(format!("{}/payments/payway/pushback", c.site_url))),
        ("cancel_url", order_url.clone()),
        ("continue_success_url", order_url),
        ("return_deeplink", String::new()),
        ("currency", "USD".into()),
        ("custom_fields", String::new()),
        ("return_params", c.order_id.to_owned()),
        ("payout", String::new()),
        ("lifetime", c.lifetime_minutes.to_string()),
        ("additional_params", String::new()),
        ("google_pay_token", String::new()),
        ("skip_success_page", String::new()),
    ];
    let hash = sign(&config.api_key, &fields.iter().map(|(_, v)| v.as_str()).collect::<Vec<_>>());
    let mut out: Vec<(&'static str, String)> = fields.into_iter().filter(|(_, v)| !v.is_empty()).collect();
    out.push(("hash", hash));
    out
}

/// Real client for `check-transaction-2`.
pub struct Client {
    pub config: Config,
    pub http: reqwest::Client,
}

impl Client {
    pub fn new(config: Config) -> Self {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(15)).build().expect("http client");
        Self { config, http }
    }
}

impl Gateway for Client {
    fn lookup<'a>(&'a self, tran_id: &'a str) -> BoxFuture<'a, anyhow::Result<Lookup>> {
        Box::pin(async move {
            let req_time = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
            let hash = sign(&self.config.api_key, &[&req_time, &self.config.merchant_id, tran_id]);
            let res = self
                .http
                .post(format!("{}/api/payment-gateway/v1/payments/check-transaction-2", self.config.api_url.trim_end_matches('/')))
                .json(&json!({ "req_time": req_time, "merchant_id": self.config.merchant_id, "tran_id": tran_id, "hash": hash }))
                .send()
                .await
                .context("payway: request")?;
            let status = res.status();
            let body: Value = res.json().await.with_context(|| format!("payway: HTTP {status}, body not JSON"))?;
            parse_check(&body)
        })
    }
}

/// Parses `check-transaction-2`: `payment_status_code` 0 = paid, 2 = pending, 3/4/7 = failed;
/// `status.code` 6 (not found: checkout not completed) = pending; other codes are errors.
pub fn parse_check(body: &Value) -> anyhow::Result<Lookup> {
    let data = &body["data"];
    // `status.code` is "00" on success and a number on errors; it sits under `data` or at the top.
    let status = if data["status"].is_object() { &data["status"]["code"] } else { &body["status"]["code"] };
    let code = status.as_str().map(str::to_owned).or_else(|| status.as_i64().map(|c| c.to_string()));
    match code.as_deref() {
        Some("00" | "0") => {}
        Some("6") => return Ok(Lookup::Pending),
        _ => anyhow::bail!("payway: {}", body["status"]["message"].as_str().or(data["status"]["message"].as_str()).unwrap_or("unexpected response")),
    }
    match data["payment_status_code"].as_i64() {
        Some(0) => {
            let amount = data["original_amount"].as_f64().or(data["total_amount"].as_f64()).context("payway: amount missing")?;
            Ok(Lookup::Paid {
                amount_cents: cents(amount),
                currency: data["payment_currency"].as_str().context("payway: currency missing")?.to_owned(),
                to_account: None,
                provider_txn: data["apv"].as_str().unwrap_or_default().to_owned(),
            })
        }
        Some(2) => Ok(Lookup::Pending),
        Some(3 | 4 | 7) => Ok(Lookup::Failed),
        other => anyhow::bail!("payway: unexpected payment_status_code {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn config() -> Config {
        Config { merchant_id: "ec000262".into(), api_key: "test-api-key".into(), api_url: "https://checkout-sandbox.payway.com.kh".into() }
    }

    #[test]
    fn signs_like_the_reference_hmac_sha512() {
        // Reference values computed independently with Python's hmac/hashlib.
        assert_eq!(
            sign("test-api-key", &["20261104093000", "ec000262", "0123456789abcdef0123"]),
            "EvDM8vO6uhT9n++aqqpDHO88IPLabGVUuDAmCsvhLHRmHxiH0UJgjdmZ87OQrM5HARFpwsyE7o4yAXc0926Rbw=="
        );
    }

    #[test]
    fn checkout_fields_are_signed_in_the_documented_order() {
        let items = [("X".to_owned(), 1550)];
        let c = Checkout {
            req_time: "20261104093000",
            tran_id: "0123456789abcdef0123",
            amount_cents: 1550,
            items: &items,
            firstname: "Sok",
            order_id: "order-1",
            site_url: "https://example.com",
            lifetime_minutes: 30,
        };
        let fields = checkout_fields(&config(), &c);
        let get = |k: &str| fields.iter().find(|(name, _)| *name == k).map(|(_, v)| v.as_str());
        assert_eq!(get("hash"), Some("8d6VxxHBtlH303UmqZWHDiyrKSHWPvekjTVuNkBMQE7yaWbEJU76r2qO9ErLZiXybg1LnYNqWveO4Yj5P0iudQ=="));
        assert_eq!(get("amount"), Some("15.50"));
        assert_eq!(get("currency"), Some("USD"));
        assert_eq!(get("merchant_id"), Some("ec000262"));
        assert_eq!(get("return_url"), Some("aHR0cHM6Ly9leGFtcGxlLmNvbS9wYXltZW50cy9wYXl3YXkvcHVzaGJhY2s="));
        assert_eq!(get("continue_success_url"), Some("https://example.com/orders/order-1"));
        assert_eq!(get("lastname"), None);
    }

    #[test]
    fn parses_the_documented_responses() {
        let ok = json!({"data": {"payment_status_code": 0, "total_amount": 0.1, "original_amount": 0.1, "refund_amount": 0,
            "discount_amount": 0, "payment_amount": 0.1, "payment_currency": "USD", "apv": "753786", "payment_status": "APPROVED",
            "transaction_date": "2025-02-13 13:55:25", "status": {"code": "00", "message": "Success!", "tran_id": "17394277693"}}});
        assert_eq!(
            parse_check(&ok).unwrap(),
            Lookup::Paid { amount_cents: 10, currency: "USD".into(), to_account: None, provider_txn: "753786".into() }
        );
        let not_found = json!({"status": {"code": 6, "message": "Transaction not found", "tran_id": "1729573626"}});
        assert_eq!(parse_check(&not_found).unwrap(), Lookup::Pending);
        let mut pending = ok.clone();
        pending["data"]["payment_status_code"] = json!(2);
        assert_eq!(parse_check(&pending).unwrap(), Lookup::Pending);
        for code in [3, 4, 7] {
            let mut failed = ok.clone();
            failed["data"]["payment_status_code"] = json!(code);
            assert_eq!(parse_check(&failed).unwrap(), Lookup::Failed);
        }
        assert!(parse_check(&json!({"status": {"code": 5, "message": "Invalid hash"}})).is_err());
    }
}
