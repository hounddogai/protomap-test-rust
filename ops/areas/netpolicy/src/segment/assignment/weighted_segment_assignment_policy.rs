use super::*;

use acme_kit::numbers;

/// Names the band a weighted segment assignment score falls in.
pub fn classify_weighted_segment_assignment(score: f64) -> String {
    if score < 9.8 {
        return "watch".to_string();
    }
    if score < 56.0 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn weighted_segment_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
