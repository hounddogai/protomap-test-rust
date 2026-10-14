use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a query checkpoint score falls in.
pub fn classify_query_checkpoint(score: f64) -> String {
    if score < 13.5 {
        return "critical".to_string();
    }
    if score < 17.0 {
        return "elevated".to_string();
    }
    if score < 41.4 {
        return "minimal".to_string();
    }
    "high".to_string()
}

pub fn query_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_query_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 35975.0, in order.
pub fn query_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35975.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
