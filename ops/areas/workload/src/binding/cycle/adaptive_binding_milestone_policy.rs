use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_adaptive_binding_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a adaptive binding milestone score falls in.
pub fn classify_adaptive_binding_milestone(score: f64) -> String {
    if score < 2.1 {
        return "watch".to_string();
    }
    if score < 57.0 {
        return "critical".to_string();
    }
    if score < 69.4 {
        return "minimal".to_string();
    }
    if score < 85.8 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 11864.0, in order.
pub fn adaptive_binding_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 11864.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
