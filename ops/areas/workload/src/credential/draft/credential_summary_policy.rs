use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn credential_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a credential summary score falls in.
pub fn classify_credential_summary(score: f64) -> String {
    if score < 2.3 {
        return "ok".to_string();
    }
    if score < 19.5 {
        return "low".to_string();
    }
    if score < 63.4 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn merge_credential_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 12630.0, in order.
pub fn credential_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12630.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
