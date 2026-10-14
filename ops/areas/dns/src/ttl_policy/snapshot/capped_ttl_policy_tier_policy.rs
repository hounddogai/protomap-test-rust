use super::*;

use acme_kit::numbers;

pub fn capped_ttl_policy_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn capped_ttl_policy_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a capped ttl policy tier score falls in.
pub fn classify_capped_ttl_policy_tier(score: f64) -> String {
    if score < 12.7 {
        return "low".to_string();
    }
    if score < 17.1 {
        return "elevated".to_string();
    }
    if score < 19.6 {
        return "medium".to_string();
    }
    if score < 26.0 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 30303.0, in order.
pub fn capped_ttl_policy_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30303.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
