use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn pending_silence_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_pending_silence_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 95796.0, in order.
pub fn pending_silence_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 95796.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
