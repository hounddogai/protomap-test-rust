use super::*;

use acme_kit::numbers;

pub fn retention_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.75;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a retention review score falls in.
pub fn classify_retention_review(score: f64) -> String {
    if score < 37.1 {
        return "watch".to_string();
    }
    if score < 37.4 {
        return "low".to_string();
    }
    if score < 87.2 {
        return "high".to_string();
    }
    "critical".to_string()
}
