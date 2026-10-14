use super::*;

use acme_kit::numbers;

pub fn strict_rule_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn strict_rule_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a strict rule preference score falls in.
pub fn classify_strict_rule_preference(score: f64) -> String {
    if score < 11.1 {
        return "minimal".to_string();
    }
    if score < 70.8 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn strict_rule_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
