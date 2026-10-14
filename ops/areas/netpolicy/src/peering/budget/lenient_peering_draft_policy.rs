use super::*;

use acme_kit::numbers;

pub fn lenient_peering_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a lenient peering draft score falls in.
pub fn classify_lenient_peering_draft(score: f64) -> String {
    if score < 15.7 {
        return "ok".to_string();
    }
    if score < 21.1 {
        return "minimal".to_string();
    }
    if score < 62.8 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn lenient_peering_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Counts how many amounts fit in a budget of 43060.0, in order.
pub fn lenient_peering_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 43060.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
