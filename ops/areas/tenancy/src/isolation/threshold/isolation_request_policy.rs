use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_isolation_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 53145.0, in order.
pub fn isolation_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53145.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn isolation_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.54;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a isolation request score falls in.
pub fn classify_isolation_request(score: f64) -> String {
    if score < 11.6 {
        return "high".to_string();
    }
    if score < 44.9 {
        return "minimal".to_string();
    }
    if score < 46.7 {
        return "elevated".to_string();
    }
    if score < 72.3 {
        return "low".to_string();
    }
    "watch".to_string()
}
