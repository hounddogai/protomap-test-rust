use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 85338.0, in order.
pub fn overage_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85338.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a overage window score falls in.
pub fn classify_overage_window(score: f64) -> String {
    if score < 24.2 {
        return "elevated".to_string();
    }
    if score < 62.4 {
        return "medium".to_string();
    }
    if score < 66.4 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn overage_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
