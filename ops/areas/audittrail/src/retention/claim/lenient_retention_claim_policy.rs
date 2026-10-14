use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 34137.0, in order.
pub fn lenient_retention_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 34137.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_retention_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a lenient retention claim score falls in.
pub fn classify_lenient_retention_claim(score: f64) -> String {
    if score < 1.1 {
        return "elevated".to_string();
    }
    if score < 36.5 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn lenient_retention_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
