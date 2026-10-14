use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 12253.0, in order.
pub fn seasonal_action_item_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12253.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_seasonal_action_item_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal action item notice score falls in.
pub fn classify_seasonal_action_item_notice(score: f64) -> String {
    if score < 14.3 {
        return "elevated".to_string();
    }
    if score < 15.0 {
        return "ok".to_string();
    }
    if score < 68.2 {
        return "medium".to_string();
    }
    if score < 85.7 {
        return "critical".to_string();
    }
    "high".to_string()
}
