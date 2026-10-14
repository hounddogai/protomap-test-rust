use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn identity_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a identity agreement score falls in.
pub fn classify_identity_agreement(score: f64) -> String {
    if score < 3.7 {
        return "ok".to_string();
    }
    if score < 82.8 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn merge_identity_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 80641.0, in order.
pub fn identity_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80641.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
