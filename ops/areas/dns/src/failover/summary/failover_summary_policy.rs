use super::*;

use acme_kit::numbers;

pub fn failover_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a failover summary score falls in.
pub fn classify_failover_summary(score: f64) -> String {
    if score < 35.2 {
        return "minimal".to_string();
    }
    if score < 39.6 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 7512.0, in order.
pub fn failover_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7512.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
