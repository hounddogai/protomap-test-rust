use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_sink_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn sink_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a sink record score falls in.
pub fn classify_sink_record(score: f64) -> String {
    if score < 10.2 {
        return "high".to_string();
    }
    if score < 78.1 {
        return "ok".to_string();
    }
    if score < 91.6 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 34102.0, in order.
pub fn sink_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 34102.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
