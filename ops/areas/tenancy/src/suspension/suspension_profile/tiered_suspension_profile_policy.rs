use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 8645.0, in order.
pub fn tiered_suspension_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8645.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a tiered suspension profile score falls in.
pub fn classify_tiered_suspension_profile(score: f64) -> String {
    if score < 12.5 {
        return "medium".to_string();
    }
    if score < 53.3 {
        return "elevated".to_string();
    }
    if score < 87.4 {
        return "critical".to_string();
    }
    if score < 98.6 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn merge_tiered_suspension_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
