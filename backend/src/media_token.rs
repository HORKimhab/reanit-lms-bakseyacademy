//! Short-lived signed tokens for `/media/{token}`: which media file, until when, inline or download.
//! HMAC-SHA256 with `MEDIA_SIGNING_KEY`; the token is the whole credential, so it must expire.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MediaGrant {
    pub media_id: Uuid,
    /// Unix seconds.
    pub expires: i64,
    /// `Content-Disposition: attachment` (download) instead of inline.
    pub attachment: bool,
}

pub fn sign(key: &[u8], grant: &MediaGrant) -> String {
    let claims = format!("{}.{}.{}", grant.media_id, grant.expires, if grant.attachment { 'a' } else { 'i' });
    let sig = mac(key, claims.as_bytes()).finalize().into_bytes();
    format!("{}.{}", URL_SAFE_NO_PAD.encode(claims), URL_SAFE_NO_PAD.encode(sig))
}

/// The grant, if `token` was signed with `key` and has not expired at `now` (unix seconds).
pub fn verify(key: &[u8], token: &str, now: i64) -> Option<MediaGrant> {
    let (claims, sig) = token.split_once('.')?;
    let claims = URL_SAFE_NO_PAD.decode(claims).ok()?;
    let sig = URL_SAFE_NO_PAD.decode(sig).ok()?;
    mac(key, &claims).verify_slice(&sig).ok()?; // constant-time
    let claims = std::str::from_utf8(&claims).ok()?;
    let mut parts = claims.split('.');
    let media_id = parts.next()?.parse().ok()?;
    let expires: i64 = parts.next()?.parse().ok()?;
    let attachment = match parts.next()? {
        "a" => true,
        "i" => false,
        _ => return None,
    };
    if parts.next().is_some() || now >= expires {
        return None;
    }
    Some(MediaGrant { media_id, expires, attachment })
}

fn mac(key: &[u8], data: &[u8]) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &[u8] = b"test-key";

    fn grant() -> MediaGrant {
        MediaGrant { media_id: Uuid::from_u128(0x1234), expires: 1_000, attachment: true }
    }

    #[test]
    fn round_trips_until_expiry() {
        let token = sign(KEY, &grant());
        assert!(token.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)), "{token}");
        assert_eq!(verify(KEY, &token, 999), Some(grant()));
        assert_eq!(verify(KEY, &token, 1_000), None);
    }

    #[test]
    fn rejects_tampering_wrong_key_and_garbage() {
        let token = sign(KEY, &grant());
        assert_eq!(verify(b"other-key", &token, 0), None);

        // Same signature, different claims (later expiry / inline instead of attachment).
        let sig = token.rsplit_once('.').unwrap().1;
        for claims in [format!("{}.9999.a", grant().media_id), format!("{}.1000.i", grant().media_id)] {
            let forged = format!("{}.{sig}", URL_SAFE_NO_PAD.encode(&claims));
            assert_eq!(verify(KEY, &forged, 0), None, "{claims}");
        }
        // A signature that is valid for different claims doesn't transfer.
        let other = sign(KEY, &MediaGrant { attachment: false, ..grant() });
        let swapped = format!("{}.{}", token.split_once('.').unwrap().0, other.rsplit_once('.').unwrap().1);
        assert_eq!(verify(KEY, &swapped, 0), None);

        for junk in ["", ".", "abc", "abc.def", "!!.!!", &format!("{}.", token.split_once('.').unwrap().0)] {
            assert_eq!(verify(KEY, junk, 0), None, "{junk}");
        }
    }
}
