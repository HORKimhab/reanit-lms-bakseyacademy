//! Failed-login throttle: at most `MAX_FAILURES` wrong passwords per phone number per window.
//! In-memory (per API process): good enough to stop online guessing against one account.

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

pub const MAX_FAILURES: u32 = 10;
pub const WINDOW: Duration = Duration::from_secs(15 * 60);

pub struct LoginThrottle {
    /// key → (failures in the current window, window start)
    failures: Mutex<HashMap<String, (u32, Instant)>>,
    max: u32,
    window: Duration,
}

impl Default for LoginThrottle {
    fn default() -> Self {
        Self::with_limit(MAX_FAILURES, WINDOW)
    }
}

/// Password-reset texts: at most 3 per phone number per hour (whether or not it has an account).
pub struct ResetThrottle(pub LoginThrottle);

impl Default for ResetThrottle {
    fn default() -> Self {
        Self(LoginThrottle::with_limit(3, Duration::from_secs(60 * 60)))
    }
}

impl LoginThrottle {
    /// At most `max` recorded events per key per `window`.
    pub fn with_limit(max: u32, window: Duration) -> Self {
        Self { failures: Mutex::default(), max, window }
    }

    /// Whether `key` may try to log in at `now`.
    pub fn allowed(&self, key: &str, now: Instant) -> bool {
        let map = self.failures.lock().expect("throttle lock");
        match map.get(key) {
            Some(&(count, start)) => count < self.max || now.duration_since(start) >= self.window,
            None => true,
        }
    }

    pub fn record_failure(&self, key: &str, now: Instant) {
        let mut map = self.failures.lock().expect("throttle lock");
        // Keep memory bounded: drop finished windows once the map gets large.
        if map.len() > 10_000 {
            map.retain(|_, (_, start)| now.duration_since(*start) < self.window);
        }
        let entry = map.entry(key.to_owned()).or_insert((0, now));
        if now.duration_since(entry.1) >= self.window {
            *entry = (0, now);
        }
        entry.0 += 1;
    }

    pub fn record_success(&self, key: &str) {
        self.failures.lock().expect("throttle lock").remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_max_failures_until_the_window_ends() {
        let t = LoginThrottle::default();
        let start = Instant::now();
        for i in 0..MAX_FAILURES {
            assert!(t.allowed("+85512345678", start), "attempt {i}");
            t.record_failure("+85512345678", start);
        }
        assert!(!t.allowed("+85512345678", start + Duration::from_secs(60)));
        assert!(t.allowed("+85599999999", start), "other numbers are unaffected");
        assert!(t.allowed("+85512345678", start + WINDOW), "window over");
    }

    #[test]
    fn success_resets_the_count() {
        let t = LoginThrottle::default();
        let now = Instant::now();
        for _ in 0..MAX_FAILURES - 1 {
            t.record_failure("k", now);
        }
        t.record_success("k");
        t.record_failure("k", now);
        assert!(t.allowed("k", now));
    }
}
