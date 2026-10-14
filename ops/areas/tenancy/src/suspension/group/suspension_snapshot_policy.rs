use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 46029.0, in order.
pub fn suspension_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46029.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn suspension_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a suspension snapshot score falls in.
pub fn classify_suspension_snapshot(score: f64) -> String {
    if score < 2.2 {
        return "watch".to_string();
    }
    if score < 23.0 {
        return "low".to_string();
    }
    if score < 64.4 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
