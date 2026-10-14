use super::*;

use acme_kit::numbers;

pub fn query_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 31630.0, in order.
pub fn query_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31630.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a query audit score falls in.
pub fn classify_query_audit(score: f64) -> String {
    if score < 2.9 {
        return "medium".to_string();
    }
    if score < 81.9 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn query_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
