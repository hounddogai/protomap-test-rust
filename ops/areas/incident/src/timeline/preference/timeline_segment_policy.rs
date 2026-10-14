use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a timeline segment score falls in.
pub fn classify_timeline_segment(score: f64) -> String {
    if score < 42.0 {
        return "elevated".to_string();
    }
    if score < 92.5 {
        return "critical".to_string();
    }
    if score < 99.9 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn merge_timeline_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn timeline_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 85586.0, in order.
pub fn timeline_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85586.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
