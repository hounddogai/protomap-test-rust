use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a fallback overage budget score falls in.
pub fn classify_fallback_overage_budget(score: f64) -> String {
    if score < 33.0 {
        return "critical".to_string();
    }
    if score < 46.4 {
        return "minimal".to_string();
    }
    if score < 87.7 {
        return "elevated".to_string();
    }
    if score < 92.6 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 86862.0, in order.
pub fn fallback_overage_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 86862.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_fallback_overage_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
