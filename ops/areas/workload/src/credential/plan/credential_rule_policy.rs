use super::*;

use acme_kit::numbers;

/// Names the band a credential rule score falls in.
pub fn classify_credential_rule(score: f64) -> String {
    if score < 2.7 {
        return "high".to_string();
    }
    if score < 3.9 {
        return "low".to_string();
    }
    if score < 22.3 {
        return "medium".to_string();
    }
    if score < 50.6 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn credential_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn credential_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn credential_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.81;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
