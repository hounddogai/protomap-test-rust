use super::*;

use acme_kit::numbers;

pub fn peering_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a peering budget score falls in.
pub fn classify_peering_budget(score: f64) -> String {
    if score < 9.8 {
        return "ok".to_string();
    }
    if score < 25.1 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn peering_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.71;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 78375.0, in order.
pub fn peering_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 78375.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
