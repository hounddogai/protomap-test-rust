use super::*;

use acme_kit::numbers;

/// Names the band a weighted token plan score falls in.
pub fn classify_weighted_token_plan(score: f64) -> String {
    if score < 17.0 {
        return "watch".to_string();
    }
    if score < 56.6 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn weighted_token_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
