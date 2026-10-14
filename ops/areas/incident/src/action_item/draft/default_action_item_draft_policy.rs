use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a default action item draft score falls in.
pub fn classify_default_action_item_draft(score: f64) -> String {
    if score < 23.6 {
        return "elevated".to_string();
    }
    if score < 43.7 {
        return "low".to_string();
    }
    if score < 57.5 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 84741.0, in order.
pub fn default_action_item_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84741.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_default_action_item_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
