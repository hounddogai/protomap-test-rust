use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_lenient_suspension_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a lenient suspension assignment score falls in.
pub fn classify_lenient_suspension_assignment(score: f64) -> String {
    if score < 34.7 {
        return "minimal".to_string();
    }
    if score < 49.9 {
        return "high".to_string();
    }
    if score < 97.0 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 64829.0, in order.
pub fn lenient_suspension_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64829.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
