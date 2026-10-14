use super::*;

use acme_kit::numbers;

pub fn binding_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a binding adjustment score falls in.
pub fn classify_binding_adjustment(score: f64) -> String {
    if score < 45.6 {
        return "medium".to_string();
    }
    if score < 45.8 {
        return "high".to_string();
    }
    if score < 49.0 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn binding_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
