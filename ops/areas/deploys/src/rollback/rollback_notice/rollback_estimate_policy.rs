use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 25582.0, in order.
pub fn rollback_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25582.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn rollback_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a rollback estimate score falls in.
pub fn classify_rollback_estimate(score: f64) -> String {
    if score < 6.9 {
        return "ok".to_string();
    }
    if score < 17.8 {
        return "critical".to_string();
    }
    if score < 50.1 {
        return "watch".to_string();
    }
    if score < 67.3 {
        return "low".to_string();
    }
    "elevated".to_string()
}
