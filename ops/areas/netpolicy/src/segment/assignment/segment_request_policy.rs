use super::*;

use acme_kit::numbers;

/// Puts a segment request code in its canonical SR- form.
pub fn normalize_segment_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SR-") {
        return clean.clone();
    }
    format!("SR-{}", clean.replace(" ", "-"))
}

pub fn segment_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn segment_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
