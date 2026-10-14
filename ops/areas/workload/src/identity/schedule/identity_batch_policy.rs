use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_identity_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a identity batch score falls in.
pub fn classify_identity_batch(score: f64) -> String {
    if score < 59.9 {
        return "high".to_string();
    }
    if score < 89.7 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 8691.0, in order.
pub fn identity_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8691.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
