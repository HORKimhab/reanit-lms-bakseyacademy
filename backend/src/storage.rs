//! Presigned GET URLs for an S3-compatible bucket (Cloudflare R2, AWS S3, MinIO), AWS SigV4.

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct ObjectStorage {
    /// e.g. `https://<account>.r2.cloudflarestorage.com` or `http://127.0.0.1:9000`
    pub endpoint: String,
    pub bucket: String,
    /// `auto` for R2.
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    /// `endpoint/bucket/key` (true) or `bucket.endpoint/key`.
    pub path_style: bool,
}

impl ObjectStorage {
    /// A URL that lets anyone holding it GET `key` until `now + expires_secs`.
    /// `query` adds signed request parameters such as `response-content-disposition`.
    pub fn presign_get(&self, key: &str, query: &[(&str, &str)], now: DateTime<Utc>, expires_secs: u32) -> String {
        let (scheme, endpoint_host) = self.endpoint.split_once("://").unwrap_or(("https", &self.endpoint));
        let endpoint_host = endpoint_host.trim_end_matches('/');
        let key = uri_encode(key.trim_start_matches('/'), false);
        let (host, path) = if self.path_style {
            (endpoint_host.to_owned(), format!("/{}/{key}", uri_encode(&self.bucket, true)))
        } else {
            (format!("{}.{endpoint_host}", self.bucket), format!("/{key}"))
        };

        let date = now.format("%Y%m%d").to_string();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();
        let scope = format!("{date}/{}/s3/aws4_request", self.region);
        let credential = format!("{}/{scope}", self.access_key_id);
        let expires = expires_secs.to_string();

        let mut params: Vec<(String, String)> = [
            ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
            ("X-Amz-Credential", credential.as_str()),
            ("X-Amz-Date", timestamp.as_str()),
            ("X-Amz-Expires", expires.as_str()),
            ("X-Amz-SignedHeaders", "host"),
        ]
        .iter()
        .chain(query)
        .map(|(k, v)| (uri_encode(k, true), uri_encode(v, true)))
        .collect();
        params.sort();
        let canonical_query = params.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&");

        let canonical_request = format!("GET\n{path}\n{canonical_query}\nhost:{host}\n\nhost\nUNSIGNED-PAYLOAD");
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{timestamp}\n{scope}\n{:x}",
            Sha256::digest(canonical_request.as_bytes())
        );
        let mut signing_key = format!("AWS4{}", self.secret_access_key).into_bytes();
        for part in [date.as_str(), &self.region, "s3", "aws4_request"] {
            signing_key = hmac(&signing_key, part.as_bytes());
        }
        let signature: String = hmac(&signing_key, string_to_sign.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();

        format!("{scheme}://{host}{path}?{canonical_query}&X-Amz-Signature={signature}")
    }
}

fn hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// SigV4 URI encoding: everything but `A-Za-z0-9-_.~` is percent-encoded; `/` too unless in a path.
fn uri_encode(s: &str, encode_slash: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b'/' if !encode_slash => out.push('/'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    /// AWS's published example: "Authenticating Requests: Using Query Parameters (AWS Signature Version 4)".
    #[test]
    fn matches_aws_documented_presigned_url() {
        let s3 = ObjectStorage {
            endpoint: "https://s3.amazonaws.com".into(),
            bucket: "examplebucket".into(),
            region: "us-east-1".into(),
            access_key_id: "AKIAIOSFODNN7EXAMPLE".into(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".into(),
            path_style: false,
        };
        let now = Utc.with_ymd_and_hms(2013, 5, 24, 0, 0, 0).unwrap();
        assert_eq!(
            s3.presign_get("test.txt", &[], now, 86400),
            "https://examplebucket.s3.amazonaws.com/test.txt\
             ?X-Amz-Algorithm=AWS4-HMAC-SHA256\
             &X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request\
             &X-Amz-Date=20130524T000000Z&X-Amz-Expires=86400&X-Amz-SignedHeaders=host\
             &X-Amz-Signature=aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404"
        );
    }

    /// Cross-checked against boto3 `generate_presigned_url` (path style, space in key,
    /// `ResponseContentDisposition`); boto orders query params differently, so compare the parts.
    #[test]
    fn path_style_with_disposition_matches_boto3() {
        let s3 = ObjectStorage {
            endpoint: "http://127.0.0.1:9010".into(),
            bucket: "ebooks".into(),
            region: "auto".into(),
            access_key_id: "AKID".into(),
            secret_access_key: "SECRET".into(),
            path_style: true,
        };
        let now = Utc.with_ymd_and_hms(2026, 11, 2, 7, 25, 34).unwrap();
        let url = s3.presign_get(
            "books/personal finance.pdf",
            &[("response-content-disposition", "attachment; filename=\"personal-finance.pdf\"")],
            now,
            3600,
        );
        assert!(url.starts_with("http://127.0.0.1:9010/ebooks/books/personal%20finance.pdf?"), "{url}");
        assert!(
            url.contains("response-content-disposition=attachment%3B%20filename%3D%22personal-finance.pdf%22"),
            "{url}"
        );
        assert!(
            url.ends_with("&X-Amz-Signature=8e69f7ec29112e37b1f0b0dd3fda8a1ef733c9d77fa803d7938ad34360af8cec"),
            "{url}"
        );
    }
}
