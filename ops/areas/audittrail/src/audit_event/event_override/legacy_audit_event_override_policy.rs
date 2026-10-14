use super::*;

use acme_kit::numbers;

/// Names the band a legacy audit event override score falls in.
pub fn classify_legacy_audit_event_override(score: f64) -> String {
    if score < 25.3 {
        return "watch".to_string();
    }
    if score < 38.1 {
        return "high".to_string();
    }
    if score < 70.1 {
        return "ok".to_string();
    }
    if score < 88.2 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

pub fn legacy_audit_event_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
