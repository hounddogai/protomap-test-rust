use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_pending_action_item_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn pending_action_item_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a pending action item entry score falls in.
pub fn classify_pending_action_item_entry(score: f64) -> String {
    if score < 40.7 {
        return "minimal".to_string();
    }
    if score < 46.3 {
        return "elevated".to_string();
    }
    if score < 57.7 {
        return "high".to_string();
    }
    if score < 70.6 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 96685.0, in order.
pub fn pending_action_item_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 96685.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
