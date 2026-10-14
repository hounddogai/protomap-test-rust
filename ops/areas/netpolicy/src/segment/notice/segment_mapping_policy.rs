use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 44017.0, in order.
pub fn segment_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 44017.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn segment_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn segment_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn merge_segment_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
