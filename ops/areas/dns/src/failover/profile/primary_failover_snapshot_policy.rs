use super::*;

use acme_kit::numbers;

pub fn primary_failover_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a primary failover snapshot score falls in.
pub fn classify_primary_failover_snapshot(score: f64) -> String {
    if score < 29.4 {
        return "high".to_string();
    }
    if score < 49.3 {
        return "elevated".to_string();
    }
    if score < 78.4 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 87648.0, in order.
pub fn primary_failover_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87648.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
