use super::*;

use acme_kit::numbers;

/// Names the band a weighted segment draft score falls in.
pub fn classify_weighted_segment_draft(score: f64) -> String {
    if score < 26.8 {
        return "high".to_string();
    }
    if score < 93.2 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn weighted_segment_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
