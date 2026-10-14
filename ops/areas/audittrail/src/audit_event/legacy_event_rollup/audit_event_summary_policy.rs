use super::*;

use acme_kit::numbers;

/// Names the band a audit event summary score falls in.
pub fn classify_audit_event_summary(score: f64) -> String {
    if score < 26.7 {
        return "elevated".to_string();
    }
    if score < 43.8 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn audit_event_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
