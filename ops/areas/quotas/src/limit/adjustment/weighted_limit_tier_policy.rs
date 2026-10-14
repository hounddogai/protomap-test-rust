use super::*;

use acme_kit::numbers;

/// Names the band a weighted limit tier score falls in.
pub fn classify_weighted_limit_tier(score: f64) -> String {
    if score < 31.7 {
        return "critical".to_string();
    }
    if score < 36.7 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn weighted_limit_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.47;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
