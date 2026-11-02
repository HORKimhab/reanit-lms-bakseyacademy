//! HTTP `Range` requests (single byte ranges), for streaming files from disk.

/// `Range: bytes=…` → inclusive `(start, end)` within a file of `total` bytes.
/// `Ok(None)`: no header, or one we ignore (malformed / multi-range) → send the whole file.
/// `Err(())`: syntactically fine but unsatisfiable → 416.
pub fn parse_range(header: Option<&str>, total: u64) -> Result<Option<(u64, u64)>, ()> {
    let Some(spec) = header.and_then(|h| h.trim().strip_prefix("bytes=")) else {
        return Ok(None);
    };
    if spec.contains(',') {
        return Ok(None);
    }
    let Some((start, end)) = spec.split_once('-') else { return Ok(None) };
    let num = |s: &str| -> Option<u64> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) { None } else { s.parse().ok() }
    };

    // bytes=-N: the last N bytes
    if start.is_empty() {
        let Some(n) = num(end) else { return Ok(None) };
        if n == 0 || total == 0 {
            return Err(());
        }
        return Ok(Some((total.saturating_sub(n), total - 1)));
    }

    // bytes=A- / bytes=A-B
    let Some(a) = num(start) else { return Ok(None) };
    let b = match end {
        "" => None,
        e => match num(e) {
            Some(b) if b >= a => Some(b),
            _ => return Ok(None),
        },
    };
    if a >= total {
        return Err(());
    }
    Ok(Some((a, b.map_or(total - 1, |b| b.min(total - 1)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges() {
        assert_eq!(parse_range(None, 1000), Ok(None));
        assert_eq!(parse_range(Some("bytes=0-99"), 1000), Ok(Some((0, 99))));
        assert_eq!(parse_range(Some("bytes=900-"), 1000), Ok(Some((900, 999))));
        assert_eq!(parse_range(Some("bytes=-100"), 1000), Ok(Some((900, 999))));
        // end past the file is clamped; suffix longer than the file is the whole file
        assert_eq!(parse_range(Some("bytes=990-5000"), 1000), Ok(Some((990, 999))));
        assert_eq!(parse_range(Some("bytes=-5000"), 1000), Ok(Some((0, 999))));
        // ignored → whole file
        for ignored in ["items=0-1", "bytes=a-b", "bytes=5-1", "bytes=0-1,5-6", "bytes=-"] {
            assert_eq!(parse_range(Some(ignored), 1000), Ok(None), "{ignored}");
        }
        // unsatisfiable
        assert_eq!(parse_range(Some("bytes=1000-"), 1000), Err(()));
        assert_eq!(parse_range(Some("bytes=-0"), 1000), Err(()));
        assert_eq!(parse_range(Some("bytes=0-"), 0), Err(()));
    }
}
