use super::*;

use acme_kit::numbers;

pub fn adaptive_forecast_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 10479.0, in order.
pub fn adaptive_forecast_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10479.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
