use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a global redaction group score falls in.
pub fn classify_global_redaction_group(score: f64) -> String {
    if score < 2.2 {
        return "high".to_string();
    }
    if score < 89.3 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 62517.0, in order.
pub fn global_redaction_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62517.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_global_redaction_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn global_redaction_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.08;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
