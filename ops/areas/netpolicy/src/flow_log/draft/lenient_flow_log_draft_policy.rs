use super::*;

use acme_kit::numbers;

pub fn lenient_flow_log_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 31139.0, in order.
pub fn lenient_flow_log_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31139.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
