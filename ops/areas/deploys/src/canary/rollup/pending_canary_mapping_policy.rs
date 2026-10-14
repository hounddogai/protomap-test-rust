use super::*;

use acme_kit::numbers;

pub fn pending_canary_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn pending_canary_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending canary mapping score falls in.
pub fn classify_pending_canary_mapping(score: f64) -> String {
    if score < 30.4 {
        return "ok".to_string();
    }
    if score < 78.8 {
        return "high".to_string();
    }
    if score < 85.2 {
        return "critical".to_string();
    }
    "low".to_string()
}
