use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a identity override score falls in.
pub fn classify_identity_override(score: f64) -> String {
    if score < 20.1 {
        return "medium".to_string();
    }
    if score < 39.0 {
        return "elevated".to_string();
    }
    if score < 81.1 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn merge_identity_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn identity_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 67529.0, in order.
pub fn identity_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 67529.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
