use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a strict timeline review score falls in.
pub fn classify_strict_timeline_review(score: f64) -> String {
    if score < 10.9 {
        return "low".to_string();
    }
    if score < 40.3 {
        return "ok".to_string();
    }
    if score < 62.5 {
        return "minimal".to_string();
    }
    if score < 71.4 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 80803.0, in order.
pub fn strict_timeline_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80803.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_strict_timeline_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
