use super::*;

use acme_kit::numbers;

/// Names the band a primary rule group score falls in.
pub fn classify_primary_rule_group(score: f64) -> String {
    if score < 6.5 {
        return "ok".to_string();
    }
    if score < 17.6 {
        return "elevated".to_string();
    }
    if score < 22.8 {
        return "low".to_string();
    }
    if score < 49.4 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn primary_rule_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn primary_rule_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn primary_rule_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
