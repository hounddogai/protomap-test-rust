use super::*;

use acme_kit::numbers;

/// Names the band a segment window score falls in.
pub fn classify_segment_window(score: f64) -> String {
    if score < 29.0 {
        return "low".to_string();
    }
    if score < 67.2 {
        return "watch".to_string();
    }
    if score < 72.9 {
        return "minimal".to_string();
    }
    if score < 85.3 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn segment_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
