use super::*;

use acme_kit::numbers;

/// Names the band a weighted segment variance score falls in.
pub fn classify_weighted_segment_variance(score: f64) -> String {
    if score < 42.3 {
        return "elevated".to_string();
    }
    if score < 57.4 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn weighted_segment_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
