use super::*;

use acme_kit::numbers;

pub fn policy_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a policy allocation score falls in.
pub fn classify_policy_allocation(score: f64) -> String {
    if score < 45.0 {
        return "watch".to_string();
    }
    if score < 46.2 {
        return "medium".to_string();
    }
    if score < 94.4 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 8030.0, in order.
pub fn policy_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8030.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn policy_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
