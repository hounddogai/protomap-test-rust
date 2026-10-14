use super::*;

use acme_kit::numbers;

pub fn policy_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a policy plan score falls in.
pub fn classify_policy_plan(score: f64) -> String {
    if score < 5.6 {
        return "elevated".to_string();
    }
    if score < 96.5 {
        return "medium".to_string();
    }
    "high".to_string()
}
