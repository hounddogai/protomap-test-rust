use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_nonce_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 32801.0, in order.
pub fn nonce_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 32801.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
