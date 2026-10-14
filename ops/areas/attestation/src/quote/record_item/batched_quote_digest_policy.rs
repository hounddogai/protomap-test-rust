use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn batched_quote_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn merge_batched_quote_digest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a batched quote digest score falls in.
pub fn classify_batched_quote_digest(score: f64) -> String {
    if score < 14.0 {
        return "critical".to_string();
    }
    if score < 74.8 {
        return "high".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 22249.0, in order.
pub fn batched_quote_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 22249.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
