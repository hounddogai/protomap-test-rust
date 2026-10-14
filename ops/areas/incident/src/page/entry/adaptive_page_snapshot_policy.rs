use super::*;

use acme_kit::numbers;

pub fn adaptive_page_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a adaptive page snapshot score falls in.
pub fn classify_adaptive_page_snapshot(score: f64) -> String {
    if score < 14.9 {
        return "ok".to_string();
    }
    if score < 87.7 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 45135.0, in order.
pub fn adaptive_page_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45135.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
