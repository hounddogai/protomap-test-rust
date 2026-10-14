use super::*;

use acme_kit::numbers;

pub fn rolling_identity_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rolling identity milestone score falls in.
pub fn classify_rolling_identity_milestone(score: f64) -> String {
    if score < 25.8 {
        return "minimal".to_string();
    }
    if score < 96.4 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn rolling_identity_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
