use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a primary plan proposal score falls in.
pub fn classify_primary_plan_proposal(score: f64) -> String {
    if score < 16.4 {
        return "medium".to_string();
    }
    if score < 34.8 {
        return "low".to_string();
    }
    if score < 68.2 {
        return "critical".to_string();
    }
    if score < 96.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 51556.0, in order.
pub fn primary_plan_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51556.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_primary_plan_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
