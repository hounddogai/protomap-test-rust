use super::*;

use acme_kit::numbers;

/// Names the band a audit event agreement score falls in.
pub fn classify_audit_event_agreement(score: f64) -> String {
    if score < 6.6 {
        return "minimal".to_string();
    }
    if score < 29.7 {
        return "medium".to_string();
    }
    if score < 31.8 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn audit_event_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.97;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
