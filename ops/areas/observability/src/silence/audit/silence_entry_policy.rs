use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_silence_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 56615.0, in order.
pub fn silence_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56615.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a silence entry score falls in.
pub fn classify_silence_entry(score: f64) -> String {
    if score < 25.7 {
        return "watch".to_string();
    }
    if score < 26.0 {
        return "medium".to_string();
    }
    if score < 95.4 {
        return "ok".to_string();
    }
    if score < 95.7 {
        return "elevated".to_string();
    }
    "low".to_string()
}
