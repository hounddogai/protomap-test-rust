use super::*;

use acme_kit::numbers;

pub fn region_pin_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn region_pin_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a region pin quota score falls in.
pub fn classify_region_pin_quota(score: f64) -> String {
    if score < 49.5 {
        return "high".to_string();
    }
    if score < 69.4 {
        return "minimal".to_string();
    }
    if score < 72.9 {
        return "elevated".to_string();
    }
    if score < 96.0 {
        return "medium".to_string();
    }
    "low".to_string()
}
