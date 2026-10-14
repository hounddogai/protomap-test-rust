use super::*;

use acme_kit::numbers;

pub fn default_retention_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn default_retention_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a default retention checkpoint code in its canonical DRC- form.
pub fn normalize_default_retention_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRC-") {
        return clean.clone();
    }
    format!("DRC-{}", clean.replace(" ", "-"))
}
