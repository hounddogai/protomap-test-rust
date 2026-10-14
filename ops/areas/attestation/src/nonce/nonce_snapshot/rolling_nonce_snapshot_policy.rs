use super::*;

pub fn rolling_nonce_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a rolling nonce snapshot code in its canonical RNS- form.
pub fn normalize_rolling_nonce_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RNS-") {
        return clean.clone();
    }
    format!("RNS-{}", clean.replace(" ", "-"))
}
