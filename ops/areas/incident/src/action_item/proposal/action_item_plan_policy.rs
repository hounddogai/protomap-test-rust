use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 80669.0, in order.
pub fn action_item_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80669.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a action item plan score falls in.
pub fn classify_action_item_plan(score: f64) -> String {
    if score < 25.5 {
        return "low".to_string();
    }
    if score < 82.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn action_item_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.87;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
