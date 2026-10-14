use super::*;

use acme_kit::numbers;

pub fn weighted_limit_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn weighted_limit_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a weighted limit audit score falls in.
pub fn classify_weighted_limit_audit(score: f64) -> String {
    if score < 36.0 {
        return "low".to_string();
    }
    if score < 63.9 {
        return "medium".to_string();
    }
    if score < 94.9 {
        return "critical".to_string();
    }
    "watch".to_string()
}
