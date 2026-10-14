use super::*;

use acme_kit::numbers;

/// Names the band a primary policy rollup score falls in.
pub fn classify_primary_policy_rollup(score: f64) -> String {
    if score < 27.8 {
        return "medium".to_string();
    }
    if score < 89.6 {
        return "minimal".to_string();
    }
    if score < 90.3 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn primary_policy_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn primary_policy_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn primary_policy_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
