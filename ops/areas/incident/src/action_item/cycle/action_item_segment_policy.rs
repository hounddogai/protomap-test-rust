use super::*;

use acme_kit::numbers;

/// Names the band a action item segment score falls in.
pub fn classify_action_item_segment(score: f64) -> String {
    if score < 10.6 {
        return "ok".to_string();
    }
    if score < 22.0 {
        return "watch".to_string();
    }
    if score < 70.5 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn action_item_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.71;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
