use super::*;

use acme_kit::numbers;

/// Names the band a fallback alert rule score falls in.
pub fn classify_fallback_alert_rule(score: f64) -> String {
    if score < 7.3 {
        return "low".to_string();
    }
    if score < 30.1 {
        return "critical".to_string();
    }
    if score < 31.3 {
        return "elevated".to_string();
    }
    if score < 64.0 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn fallback_alert_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.88;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
