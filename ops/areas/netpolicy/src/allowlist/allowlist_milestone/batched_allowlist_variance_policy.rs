use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a batched allowlist variance score falls in.
pub fn classify_batched_allowlist_variance(score: f64) -> String {
    if score < 9.5 {
        return "ok".to_string();
    }
    if score < 40.5 {
        return "low".to_string();
    }
    if score < 49.4 {
        return "elevated".to_string();
    }
    if score < 91.9 {
        return "critical".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 93153.0, in order.
pub fn batched_allowlist_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93153.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_batched_allowlist_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn batched_allowlist_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
