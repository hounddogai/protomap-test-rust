use super::*;

use acme_kit::numbers;

/// Names the band a probe group score falls in.
pub fn classify_probe_group(score: f64) -> String {
    if score < 35.5 {
        return "elevated".to_string();
    }
    if score < 36.4 {
        return "watch".to_string();
    }
    if score < 44.5 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn probe_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 53013.0, in order.
pub fn probe_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53013.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
