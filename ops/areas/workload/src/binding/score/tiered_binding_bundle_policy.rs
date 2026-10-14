use super::*;

use acme_kit::numbers;

/// Names the band a tiered binding bundle score falls in.
pub fn classify_tiered_binding_bundle(score: f64) -> String {
    if score < 1.0 {
        return "medium".to_string();
    }
    if score < 18.9 {
        return "minimal".to_string();
    }
    if score < 71.1 {
        return "critical".to_string();
    }
    if score < 87.5 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn tiered_binding_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn tiered_binding_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
