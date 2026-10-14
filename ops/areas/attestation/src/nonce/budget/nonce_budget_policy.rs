use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 55428.0, in order.
pub fn nonce_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55428.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_nonce_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a nonce budget score falls in.
pub fn classify_nonce_budget(score: f64) -> String {
    if score < 48.9 {
        return "critical".to_string();
    }
    if score < 56.0 {
        return "minimal".to_string();
    }
    if score < 65.8 {
        return "high".to_string();
    }
    "medium".to_string()
}
