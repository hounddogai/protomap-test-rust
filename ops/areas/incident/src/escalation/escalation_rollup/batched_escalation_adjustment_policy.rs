use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a batched escalation adjustment score falls in.
pub fn classify_batched_escalation_adjustment(score: f64) -> String {
    if score < 8.1 {
        return "minimal".to_string();
    }
    if score < 36.7 {
        return "high".to_string();
    }
    if score < 50.4 {
        return "critical".to_string();
    }
    if score < 63.7 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn merge_batched_escalation_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 87386.0, in order.
pub fn batched_escalation_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87386.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
