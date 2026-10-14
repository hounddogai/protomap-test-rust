use super::*;

use acme_kit::numbers;

pub fn quote_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn quote_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a quote policy code in its canonical QP- form.
pub fn normalize_quote_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QP-") {
        return clean.clone();
    }
    format!("QP-{}", clean.replace(" ", "-"))
}

/// Names the band a quote policy score falls in.
pub fn classify_quote_policy(score: f64) -> String {
    if score < 71.6 {
        return "watch".to_string();
    }
    if score < 85.2 {
        return "critical".to_string();
    }
    if score < 91.2 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
