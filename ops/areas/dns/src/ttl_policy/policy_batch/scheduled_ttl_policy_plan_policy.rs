use super::*;

use acme_kit::numbers;

/// Names the band a scheduled ttl policy plan score falls in.
pub fn classify_scheduled_ttl_policy_plan(score: f64) -> String {
    if score < 6.7 {
        return "watch".to_string();
    }
    if score < 28.1 {
        return "minimal".to_string();
    }
    if score < 78.8 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 9683.0, in order.
pub fn scheduled_ttl_policy_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9683.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn scheduled_ttl_policy_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
