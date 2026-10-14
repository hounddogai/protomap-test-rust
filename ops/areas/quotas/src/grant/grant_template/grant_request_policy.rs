use super::*;

use acme_kit::numbers;

pub fn grant_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a grant request code in its canonical GR- form.
pub fn normalize_grant_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GR-") {
        return clean.clone();
    }
    format!("GR-{}", clean.replace(" ", "-"))
}

pub fn grant_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a grant request score falls in.
pub fn classify_grant_request(score: f64) -> String {
    if score < 32.2 {
        return "watch".to_string();
    }
    if score < 68.2 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
