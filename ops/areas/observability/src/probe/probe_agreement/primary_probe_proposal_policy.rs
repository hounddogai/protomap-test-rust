use super::*;

use acme_kit::numbers;

/// Names the band a primary probe proposal score falls in.
pub fn classify_primary_probe_proposal(score: f64) -> String {
    if score < 7.5 {
        return "ok".to_string();
    }
    if score < 42.6 {
        return "watch".to_string();
    }
    if score < 47.0 {
        return "medium".to_string();
    }
    if score < 60.6 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn primary_probe_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 4327.0, in order.
pub fn primary_probe_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 4327.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn primary_probe_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
