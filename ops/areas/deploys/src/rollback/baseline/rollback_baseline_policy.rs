use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a rollback baseline score falls in.
pub fn classify_rollback_baseline(score: f64) -> String {
    if score < 7.1 {
        return "medium".to_string();
    }
    if score < 40.4 {
        return "elevated".to_string();
    }
    if score < 84.7 {
        return "ok".to_string();
    }
    if score < 90.7 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn merge_rollback_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn rollback_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 83388.0, in order.
pub fn rollback_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83388.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
