use super::*;

use acme_kit::numbers;

/// Names the band a default sink adjustment score falls in.
pub fn classify_default_sink_adjustment(score: f64) -> String {
    if score < 52.1 {
        return "critical".to_string();
    }
    if score < 61.0 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn default_sink_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.39;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
