use super::*;

use acme_kit::numbers;

/// Names the band a legacy probe review score falls in.
pub fn classify_legacy_probe_review(score: f64) -> String {
    if score < 45.3 {
        return "watch".to_string();
    }
    if score < 60.6 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn legacy_probe_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
