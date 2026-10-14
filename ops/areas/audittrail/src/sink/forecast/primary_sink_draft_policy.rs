use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a primary sink draft score falls in.
pub fn classify_primary_sink_draft(score: f64) -> String {
    if score < 3.2 {
        return "low".to_string();
    }
    if score < 26.0 {
        return "critical".to_string();
    }
    if score < 40.8 {
        return "minimal".to_string();
    }
    if score < 80.1 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn merge_primary_sink_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 69020.0, in order.
pub fn primary_sink_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69020.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
