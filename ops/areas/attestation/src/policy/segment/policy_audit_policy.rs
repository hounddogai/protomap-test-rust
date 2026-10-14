use super::*;

use acme_kit::numbers;

pub fn policy_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a policy audit score falls in.
pub fn classify_policy_audit(score: f64) -> String {
    if score < 28.4 {
        return "high".to_string();
    }
    if score < 54.0 {
        return "medium".to_string();
    }
    if score < 56.8 {
        return "watch".to_string();
    }
    if score < 88.9 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn policy_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
