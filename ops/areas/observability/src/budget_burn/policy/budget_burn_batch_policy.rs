use super::*;

use acme_kit::numbers;

/// Names the band a budget burn batch score falls in.
pub fn classify_budget_burn_batch(score: f64) -> String {
    if score < 12.5 {
        return "low".to_string();
    }
    if score < 20.7 {
        return "high".to_string();
    }
    if score < 70.8 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn budget_burn_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn budget_burn_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
