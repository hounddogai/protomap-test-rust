use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_batched_segment_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a batched segment quota score falls in.
pub fn classify_batched_segment_quota(score: f64) -> String {
    if score < 17.1 {
        return "critical".to_string();
    }
    if score < 71.1 {
        return "medium".to_string();
    }
    if score < 81.7 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 38452.0, in order.
pub fn batched_segment_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38452.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
