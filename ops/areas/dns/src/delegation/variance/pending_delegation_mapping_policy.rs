use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_pending_delegation_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a pending delegation mapping score falls in.
pub fn classify_pending_delegation_mapping(score: f64) -> String {
    if score < 21.5 {
        return "minimal".to_string();
    }
    if score < 38.6 {
        return "medium".to_string();
    }
    if score < 51.9 {
        return "critical".to_string();
    }
    if score < 88.1 {
        return "low".to_string();
    }
    "watch".to_string()
}

pub fn pending_delegation_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.73;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 45208.0, in order.
pub fn pending_delegation_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45208.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
