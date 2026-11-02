//! Khmer display formatting for templates.

/// Renders `n` with ASCII digits replaced by Khmer digits (០–៩); other characters pass through.
pub fn khmer_number(n: impl std::fmt::Display) -> String {
    n.to_string()
        .chars()
        .map(|c| match c.to_digit(10) {
            Some(d) if c.is_ascii_digit() => char::from_u32(0x17E0 + d).unwrap(),
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn khmer_number_converts_every_digit() {
        assert_eq!(khmer_number(1234567890), "១២៣៤៥៦៧៨៩០");
    }
}
