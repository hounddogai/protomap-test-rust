use super::*;

use acme_kit::numbers;

pub fn sink_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a sink exception score falls in.
pub fn classify_sink_exception(score: f64) -> String {
    if score < 44.9 {
        return "low".to_string();
    }
    if score < 63.0 {
        return "elevated".to_string();
    }
    if score < 72.9 {
        return "watch".to_string();
    }
    if score < 74.5 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
