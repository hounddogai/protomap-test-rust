use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 66987.0, in order.
pub fn lenient_probe_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66987.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_probe_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
