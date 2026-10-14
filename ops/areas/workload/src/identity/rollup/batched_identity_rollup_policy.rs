use super::*;

use acme_kit::numbers;

/// Names the band a batched identity rollup score falls in.
pub fn classify_batched_identity_rollup(score: f64) -> String {
    if score < 15.7 {
        return "critical".to_string();
    }
    if score < 29.0 {
        return "ok".to_string();
    }
    if score < 38.9 {
        return "watch".to_string();
    }
    if score < 62.5 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 20947.0, in order.
pub fn batched_identity_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20947.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn batched_identity_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
