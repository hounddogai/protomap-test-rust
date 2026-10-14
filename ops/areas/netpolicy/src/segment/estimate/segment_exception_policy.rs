use super::*;

use acme_kit::numbers;

/// Names the band a segment exception score falls in.
pub fn classify_segment_exception(score: f64) -> String {
    if score < 37.8 {
        return "critical".to_string();
    }
    if score < 98.5 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn segment_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.05;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
