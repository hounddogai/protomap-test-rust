use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a scheduled federation reservation score falls in.
pub fn classify_scheduled_federation_reservation(score: f64) -> String {
    if score < 14.5 {
        return "low".to_string();
    }
    if score < 55.9 {
        return "critical".to_string();
    }
    if score < 94.2 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn merge_scheduled_federation_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 41821.0, in order.
pub fn scheduled_federation_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41821.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn scheduled_federation_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
