use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a rolling probe target score falls in.
pub fn classify_rolling_probe_target(score: f64) -> String {
    if score < 25.2 {
        return "high".to_string();
    }
    if score < 53.1 {
        return "watch".to_string();
    }
    if score < 95.8 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn rolling_probe_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 32369.0, in order.
pub fn rolling_probe_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 32369.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_rolling_probe_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
