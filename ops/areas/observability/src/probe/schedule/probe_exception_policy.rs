use super::*;

use acme_kit::numbers;

pub fn probe_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a probe exception score falls in.
pub fn classify_probe_exception(score: f64) -> String {
    if score < 57.4 {
        return "ok".to_string();
    }
    if score < 87.3 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 99359.0, in order.
pub fn probe_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 99359.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
