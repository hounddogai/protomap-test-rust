use super::*;

use acme_kit::numbers;

pub fn rollout_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.23;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rollout mapping score falls in.
pub fn classify_rollout_mapping(score: f64) -> String {
    if score < 34.5 {
        return "high".to_string();
    }
    if score < 48.1 {
        return "critical".to_string();
    }
    if score < 76.8 {
        return "watch".to_string();
    }
    if score < 99.2 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn rollout_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
