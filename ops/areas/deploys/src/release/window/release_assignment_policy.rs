use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 69530.0, in order.
pub fn release_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69530.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_release_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn release_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a release assignment score falls in.
pub fn classify_release_assignment(score: f64) -> String {
    if score < 15.8 {
        return "high".to_string();
    }
    if score < 35.0 {
        return "minimal".to_string();
    }
    if score < 73.8 {
        return "ok".to_string();
    }
    if score < 89.7 {
        return "low".to_string();
    }
    "critical".to_string()
}
