//! Input rules for admin-edited catalog content.

use crate::error::{AppError, AppResult};

/// Lower-case ASCII words joined by single hyphens (`rust-for-beginners`), at most 100 chars.
pub fn slug(s: &str) -> AppResult<String> {
    let s = s.trim();
    let valid = (1..=100).contains(&s.len())
        && s.split('-').all(|w| !w.is_empty() && w.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()));
    if !valid {
        return Err(AppError::Validation("invalid_slug"));
    }
    Ok(s.to_owned())
}

/// Non-negative price; a sale price must be below it.
pub fn prices(price_cents: i32, sale_price_cents: Option<i32>) -> AppResult<()> {
    let valid = price_cents >= 0 && sale_price_cents.is_none_or(|sale| (0..price_cents).contains(&sale));
    if !valid {
        return Err(AppError::Validation("invalid_price"));
    }
    Ok(())
}

/// Trimmed, non-empty, at most `max` characters.
pub fn required(s: &str, max: usize, code: &'static str) -> AppResult<String> {
    let s = s.trim();
    if s.is_empty() || s.chars().count() > max {
        return Err(AppError::Validation(code));
    }
    Ok(s.to_owned())
}

/// Trimmed; blank → `None`; at most `max` characters.
pub fn optional(s: Option<&str>, max: usize, code: &'static str) -> AppResult<Option<String>> {
    match s.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) => required(s, max, code).map(Some),
    }
}

/// Blank → `None`; otherwise a site path (`/courses`) or an `http(s)://` URL — never `javascript:` etc.
pub fn optional_url(s: Option<&str>) -> AppResult<Option<String>> {
    let Some(url) = optional(s, 2000, "invalid_url")? else {
        return Ok(None);
    };
    let site_path = url.starts_with('/') && !url.starts_with("//") && !url.starts_with("/\\");
    let http = url.starts_with("https://") || url.starts_with("http://");
    if !(site_path || http) || url.contains(|c: char| c.is_whitespace() || c.is_control()) {
        return Err(AppError::Validation("invalid_url"));
    }
    Ok(Some(url))
}

/// Maps a unique-constraint violation to `409 code` (e.g. `slug_taken`).
pub fn unique_violation(code: &'static str) -> impl FnOnce(sqlx::Error) -> AppError {
    move |e| match &e {
        sqlx::Error::Database(d) if d.is_unique_violation() => AppError::Conflict(code),
        _ => AppError::Db(e),
    }
}

/// Maps a foreign-key violation (row still referenced) to `409 in_use`.
pub fn still_referenced(e: sqlx::Error) -> AppError {
    match &e {
        sqlx::Error::Database(d) if d.is_foreign_key_violation() => AppError::Conflict("in_use"),
        _ => AppError::Db(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code<T>(r: AppResult<T>) -> &'static str {
        match r {
            Err(AppError::Validation(c)) => c,
            Err(_) => "other",
            Ok(_) => "ok",
        }
    }

    #[test]
    fn slugs_are_lowercase_hyphenated_words() {
        assert_eq!(slug(" rust-for-beginners ").unwrap(), "rust-for-beginners");
        assert_eq!(slug("web3").unwrap(), "web3");
        for bad in ["", "Rust", "a--b", "-a", "a-", "a b", "ខ្មែរ", "a_b", &"a".repeat(101)] {
            assert_eq!(code(slug(bad)), "invalid_slug", "{bad}");
        }
    }

    #[test]
    fn sale_price_must_be_below_price() {
        assert_eq!(code(prices(0, None)), "ok");
        assert_eq!(code(prices(3000, Some(1500))), "ok");
        assert_eq!(code(prices(3000, Some(0))), "ok");
        assert_eq!(code(prices(3000, Some(3000))), "invalid_price");
        assert_eq!(code(prices(-1, None)), "invalid_price");
        assert_eq!(code(prices(3000, Some(-5))), "invalid_price");
    }

    #[test]
    fn required_and_optional_text() {
        assert_eq!(required("  វគ្គ ", 10, "invalid_title").unwrap(), "វគ្គ");
        assert_eq!(code(required("   ", 10, "invalid_title")), "invalid_title");
        assert_eq!(code(required("abcdef", 5, "invalid_title")), "invalid_title");
        assert_eq!(optional(Some("  "), 5, "x").unwrap(), None);
        assert_eq!(optional(None, 5, "x").unwrap(), None);
        assert_eq!(optional(Some(" ab "), 5, "x").unwrap().as_deref(), Some("ab"));
        assert_eq!(code(optional(Some("abcdef"), 5, "too_long")), "too_long");
    }

    #[test]
    fn urls_are_site_paths_or_http() {
        assert_eq!(optional_url(Some(" /courses?category=x ")).unwrap().as_deref(), Some("/courses?category=x"));
        assert_eq!(optional_url(Some("https://cdn.example.com/a.jpg")).unwrap().as_deref(), Some("https://cdn.example.com/a.jpg"));
        assert_eq!(optional_url(Some("")).unwrap(), None);
        for bad in ["javascript:alert(1)", "//evil.com", "/\\evil.com", "data:text/html,x", "ftp://x", "https://a b", "/\t/evil.com", "/\n/evil.com", "/\u{0}/evil.com"] {
            assert_eq!(code(optional_url(Some(bad))), "invalid_url", "{bad}");
        }
    }
}
