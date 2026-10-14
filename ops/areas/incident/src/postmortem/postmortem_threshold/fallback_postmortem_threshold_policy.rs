use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_fallback_postmortem_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a fallback postmortem threshold score falls in.
pub fn classify_fallback_postmortem_threshold(score: f64) -> String {
    if score < 32.1 {
        return "high".to_string();
    }
    if score < 95.5 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

pub fn fallback_postmortem_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 45757.0, in order.
pub fn fallback_postmortem_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45757.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
