use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a capped egress review score falls in.
pub fn classify_capped_egress_review(score: f64) -> String {
    if score < 15.1 {
        return "critical".to_string();
    }
    if score < 22.2 {
        return "medium".to_string();
    }
    if score < 42.8 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn merge_capped_egress_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn capped_egress_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 48687.0, in order.
pub fn capped_egress_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48687.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
