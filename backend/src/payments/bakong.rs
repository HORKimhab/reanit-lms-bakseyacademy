//! Bakong Open API: `POST /v1/check_transaction_by_md5`.

use std::time::Duration;

use anyhow::Context;
use futures_util::future::BoxFuture;
use serde_json::{Value, json};

use super::{Gateway, Lookup, cents};

/// Real client. `token` is the Bakong Open API token (renew it before it expires, every 90 days).
pub struct Client {
    pub api_url: String,
    pub token: String,
    pub http: reqwest::Client,
}

impl Client {
    pub fn new(api_url: String, token: String) -> Self {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(15)).build().expect("http client");
        Self { api_url: api_url.trim_end_matches('/').to_owned(), token, http }
    }
}

impl Gateway for Client {
    fn lookup<'a>(&'a self, md5: &'a str) -> BoxFuture<'a, anyhow::Result<Lookup>> {
        Box::pin(async move {
            let res = self
                .http
                .post(format!("{}/v1/check_transaction_by_md5", self.api_url))
                .bearer_auth(&self.token)
                .json(&json!({ "md5": md5 }))
                .send()
                .await
                .context("bakong: request")?;
            let status = res.status();
            let body: Value = res.json().await.with_context(|| format!("bakong: HTTP {status}, body not JSON"))?;
            parse_check(&body)
        })
    }
}

/// Parses a check-by-md5 response: `responseCode 0` with `data` = paid; `errorCode 1` = not found
/// (not paid yet); anything else (expired token, …) is an error worth logging.
pub fn parse_check(body: &Value) -> anyhow::Result<Lookup> {
    match (body["responseCode"].as_i64(), body["errorCode"].as_i64()) {
        (Some(0), _) => {
            let data = &body["data"];
            let field = |k: &str| data[k].as_str().map(str::to_owned).with_context(|| format!("bakong: data.{k} missing"));
            Ok(Lookup::Paid {
                amount_cents: cents(data["amount"].as_f64().context("bakong: data.amount missing")?),
                currency: field("currency")?,
                to_account: Some(field("toAccountId")?),
                provider_txn: field("hash")?,
            })
        }
        (_, Some(1)) => Ok(Lookup::Pending),
        _ => anyhow::bail!("bakong: {}", body["responseMessage"].as_str().unwrap_or("unexpected response")),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_paid_not_found_and_errors() {
        let paid = json!({
            "responseCode": 0, "responseMessage": "Getting transaction successfully.", "errorCode": null,
            "data": {"hash": "8465d722d7d5065f2886", "fromAccountId": "payer@aba", "toAccountId": "reanit@aclb",
                     "currency": "USD", "amount": 15.5, "description": "", "createdDateMs": 1759805345337i64}
        });
        assert_eq!(
            parse_check(&paid).unwrap(),
            Lookup::Paid { amount_cents: 1550, currency: "USD".into(), to_account: Some("reanit@aclb".into()), provider_txn: "8465d722d7d5065f2886".into() }
        );
        let missing = json!({"responseCode": 1, "responseMessage": "Transaction could not be found. Please check and try again.", "errorCode": 1, "data": null});
        assert_eq!(parse_check(&missing).unwrap(), Lookup::Pending);
        let unauthorized = json!({"responseCode": 1, "responseMessage": "Unauthorized", "errorCode": 6, "data": null});
        assert!(parse_check(&unauthorized).is_err());
        assert!(parse_check(&json!({"responseCode": 0, "data": {"currency": "USD"}})).is_err());
    }
}
