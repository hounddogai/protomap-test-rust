use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_strict_isolation_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 6489.0, in order.
pub fn strict_isolation_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6489.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
