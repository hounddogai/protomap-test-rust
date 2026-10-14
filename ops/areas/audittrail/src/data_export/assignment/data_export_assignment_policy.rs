use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn data_export_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a data export assignment score falls in.
pub fn classify_data_export_assignment(score: f64) -> String {
    if score < 15.3 {
        return "elevated".to_string();
    }
    if score < 26.6 {
        return "ok".to_string();
    }
    if score < 69.3 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn merge_data_export_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 53598.0, in order.
pub fn data_export_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53598.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
