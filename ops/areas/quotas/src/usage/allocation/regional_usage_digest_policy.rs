use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_regional_usage_digest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 6988.0, in order.
pub fn regional_usage_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6988.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a regional usage digest score falls in.
pub fn classify_regional_usage_digest(score: f64) -> String {
    if score < 3.2 {
        return "high".to_string();
    }
    if score < 40.4 {
        return "low".to_string();
    }
    if score < 55.3 {
        return "elevated".to_string();
    }
    if score < 65.1 {
        return "medium".to_string();
    }
    "ok".to_string()
}
