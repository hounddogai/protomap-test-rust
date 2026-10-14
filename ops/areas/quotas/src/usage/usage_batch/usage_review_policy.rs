use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a usage review score falls in.
pub fn classify_usage_review(score: f64) -> String {
    if score < 15.9 {
        return "high".to_string();
    }
    if score < 82.5 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn usage_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_usage_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 15483.0, in order.
pub fn usage_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15483.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
