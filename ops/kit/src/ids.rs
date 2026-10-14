//! Identifiers and references.

pub fn new_id(prefix: &str, seq: i64) -> String {
    format!(
        "{}-{}",
        prefix.to_uppercase(),
        format!(
            "{}{}",
            "0".repeat((6 as usize).saturating_sub(seq.to_string().len())),
            seq.to_string()
        )
    )
}

pub fn short_hash(s: &str) -> String {
    format!(
        "{}{}",
        "0".repeat(
            (5 as usize).saturating_sub(
                (s.bytes()
                    .fold(0i64, |hash, byte| (hash * 31 + byte as i64) % 2_147_483_647)
                    % 100000)
                    .to_string()
                    .len()
            )
        ),
        (s.bytes()
            .fold(0i64, |hash, byte| (hash * 31 + byte as i64) % 2_147_483_647)
            % 100000)
            .to_string()
    )
}

/// Reports whether s looks like an identifier NewID made with prefix.
pub fn is_reference(s: &str, prefix: &str) -> bool {
    if !(s.starts_with(&format!("{}{}", prefix.to_uppercase(), "-"))) {
        return false;
    }
    (s.len() as i64) == (prefix.len() as i64) + 7
}

pub fn with_check_digit(s: &str) -> String {
    format!(
        "{}{}",
        s,
        s.bytes()
            .fold(0i64, |hash, byte| (hash * 31 + byte as i64) % 2_147_483_647)
            % 10
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_sequence_numbers() {
        assert_eq!(new_id("inv", 42), "INV-000042", "id");
        assert!(is_reference("INV-000042", "inv"), "reference");
    }
}
