use super::*;

use acme_kit::numbers;

/// Names the band a overage policy score falls in.
pub fn classify_overage_policy(score: f64) -> String {
    if score < 3.5 {
        return "medium".to_string();
    }
    if score < 73.8 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Puts a overage policy code in its canonical OP- form.
pub fn normalize_overage_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OP-") {
        return clean.clone();
    }
    format!("OP-{}", clean.replace(" ", "-"))
}

pub fn overage_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn overage_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
