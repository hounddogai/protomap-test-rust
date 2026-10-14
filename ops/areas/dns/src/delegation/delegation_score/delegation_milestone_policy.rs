use super::*;

use acme_kit::numbers;

pub fn delegation_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn delegation_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a delegation milestone score falls in.
pub fn classify_delegation_milestone(score: f64) -> String {
    if score < 16.7 {
        return "elevated".to_string();
    }
    if score < 36.9 {
        return "low".to_string();
    }
    if score < 42.7 {
        return "high".to_string();
    }
    if score < 85.1 {
        return "ok".to_string();
    }
    "critical".to_string()
}
