use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn limit_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a limit ledger score falls in.
pub fn classify_limit_ledger(score: f64) -> String {
    if score < 13.3 {
        return "minimal".to_string();
    }
    if score < 59.4 {
        return "watch".to_string();
    }
    if score < 71.8 {
        return "low".to_string();
    }
    if score < 82.6 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 28506.0, in order.
pub fn limit_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28506.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_limit_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
