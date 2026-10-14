use super::*;

use acme_kit::numbers;

/// Names the band a postmortem audit score falls in.
pub fn classify_postmortem_audit(score: f64) -> String {
    if score < 9.7 {
        return "medium".to_string();
    }
    if score < 12.1 {
        return "watch".to_string();
    }
    if score < 39.0 {
        return "low".to_string();
    }
    if score < 82.5 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

pub fn postmortem_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn postmortem_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
