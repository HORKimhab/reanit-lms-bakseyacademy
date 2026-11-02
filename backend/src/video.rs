//! Signed playback URLs for lesson videos (Bunny Stream embed-token authentication).

use sha2::{Digest, Sha256};

/// Bunny Stream library settings. With no token key the library must allow unsigned embeds.
#[derive(Clone)]
pub struct BunnyStream {
    pub library_id: String,
    pub token_key: Option<String>,
}

impl BunnyStream {
    /// Embed URL for `video_id`, valid until `expires` (unix seconds) when a token key is set.
    pub fn embed_url(&self, video_id: &str, expires: i64) -> String {
        let base = format!("https://iframe.mediadelivery.net/embed/{}/{video_id}", self.library_id);
        match &self.token_key {
            Some(key) => {
                let token = Sha256::digest(format!("{key}{video_id}{expires}").as_bytes());
                format!("{base}?token={token:x}&expires={expires}")
            }
            None => base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signs_embed_url_with_token_and_expiry() {
        let bunny = BunnyStream { library_id: "42".into(), token_key: Some("secret-key".into()) };
        assert_eq!(
            bunny.embed_url("vid-123", 1_700_000_000),
            "https://iframe.mediadelivery.net/embed/42/vid-123\
             ?token=0175e1f6a094bdf379b9e358ebec6091b3276c227bde3f88e7e46a04359f4d92&expires=1700000000"
        );
    }
}
