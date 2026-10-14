use super::*;

use acme_kit::numbers;

pub fn ttl_policy_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a ttl policy bundle score falls in.
pub fn classify_ttl_policy_bundle(score: f64) -> String {
    if score < 2.7 {
        return "minimal".to_string();
    }
    if score < 59.4 {
        return "medium".to_string();
    }
    if score < 73.1 {
        return "elevated".to_string();
    }
    if score < 95.6 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn ttl_policy_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.1;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 45903.0, in order.
pub fn ttl_policy_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45903.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
