use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 58959.0, in order.
pub fn seasonal_policy_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58959.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a seasonal policy snapshot score falls in.
pub fn classify_seasonal_policy_snapshot(score: f64) -> String {
    if score < 47.8 {
        return "ok".to_string();
    }
    if score < 59.0 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn seasonal_policy_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
