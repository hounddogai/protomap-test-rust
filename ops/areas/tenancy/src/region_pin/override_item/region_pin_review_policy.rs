use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_region_pin_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a region pin review score falls in.
pub fn classify_region_pin_review(score: f64) -> String {
    if score < 14.2 {
        return "medium".to_string();
    }
    if score < 17.9 {
        return "low".to_string();
    }
    if score < 41.0 {
        return "high".to_string();
    }
    if score < 67.0 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 11638.0, in order.
pub fn region_pin_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 11638.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
