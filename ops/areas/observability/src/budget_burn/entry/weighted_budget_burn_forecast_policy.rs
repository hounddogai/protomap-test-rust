use super::*;

/// Counts how many amounts fit in a budget of 63908.0, in order.
pub fn weighted_budget_burn_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63908.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn weighted_budget_burn_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
