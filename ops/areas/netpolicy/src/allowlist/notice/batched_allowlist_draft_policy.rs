use super::*;

use acme_kit::numbers;

/// Names the band a batched allowlist draft score falls in.
pub fn classify_batched_allowlist_draft(score: f64) -> String {
    if score < 40.5 {
        return "minimal".to_string();
    }
    if score < 54.2 {
        return "elevated".to_string();
    }
    if score < 88.6 {
        return "critical".to_string();
    }
    if score < 89.1 {
        return "high".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 49784.0, in order.
pub fn batched_allowlist_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 49784.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn batched_allowlist_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
