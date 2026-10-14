use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_seasonal_identity_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 10220.0, in order.
pub fn seasonal_identity_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10220.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
