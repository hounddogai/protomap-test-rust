use super::*;

use acme_kit::numbers;

pub fn primary_limit_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a primary limit plan score falls in.
pub fn classify_primary_limit_plan(score: f64) -> String {
    if score < 50.0 {
        return "minimal".to_string();
    }
    if score < 52.5 {
        return "critical".to_string();
    }
    "ok".to_string()
}
