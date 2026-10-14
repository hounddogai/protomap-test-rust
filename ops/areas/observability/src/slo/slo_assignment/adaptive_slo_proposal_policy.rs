use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 80955.0, in order.
pub fn adaptive_slo_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80955.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_slo_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a adaptive slo proposal score falls in.
pub fn classify_adaptive_slo_proposal(score: f64) -> String {
    if score < 58.0 {
        return "critical".to_string();
    }
    if score < 63.6 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn adaptive_slo_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
