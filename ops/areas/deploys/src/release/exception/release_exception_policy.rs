use super::*;

use acme_kit::numbers;

/// Puts a release exception code in its canonical RE- form.
pub fn normalize_release_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RE-") {
        return clean.clone();
    }
    format!("RE-{}", clean.replace(" ", "-"))
}

pub fn release_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a release exception score falls in.
pub fn classify_release_exception(score: f64) -> String {
    if score < 19.7 {
        return "minimal".to_string();
    }
    if score < 52.8 {
        return "elevated".to_string();
    }
    if score < 71.1 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn release_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
