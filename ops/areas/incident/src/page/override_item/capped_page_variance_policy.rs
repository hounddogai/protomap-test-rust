use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_capped_page_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a capped page variance score falls in.
pub fn classify_capped_page_variance(score: f64) -> String {
    if score < 29.6 {
        return "medium".to_string();
    }
    if score < 33.5 {
        return "critical".to_string();
    }
    if score < 40.5 {
        return "low".to_string();
    }
    if score < 68.8 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 85353.0, in order.
pub fn capped_page_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85353.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
