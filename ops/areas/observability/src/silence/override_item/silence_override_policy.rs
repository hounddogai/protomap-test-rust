use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a silence override score falls in.
pub fn classify_silence_override(score: f64) -> String {
    if score < 1.8 {
        return "ok".to_string();
    }
    if score < 10.5 {
        return "watch".to_string();
    }
    if score < 34.2 {
        return "low".to_string();
    }
    if score < 97.3 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn merge_silence_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn silence_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 76868.0, in order.
pub fn silence_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 76868.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
