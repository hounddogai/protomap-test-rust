use super::*;

use acme_kit::numbers;

/// Names the band a rollout milestone score falls in.
pub fn classify_rollout_milestone(score: f64) -> String {
    if score < 40.6 {
        return "ok".to_string();
    }
    if score < 81.4 {
        return "watch".to_string();
    }
    if score < 84.1 {
        return "minimal".to_string();
    }
    if score < 92.3 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn rollout_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 40616.0, in order.
pub fn rollout_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40616.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
