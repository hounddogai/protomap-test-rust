use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 73086.0, in order.
pub fn regional_suspension_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73086.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_regional_suspension_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn regional_suspension_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a regional suspension policy score falls in.
pub fn classify_regional_suspension_policy(score: f64) -> String {
    if score < 35.6 {
        return "high".to_string();
    }
    if score < 37.6 {
        return "ok".to_string();
    }
    if score < 47.1 {
        return "elevated".to_string();
    }
    if score < 51.3 {
        return "low".to_string();
    }
    "watch".to_string()
}
