use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn evidence_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.11;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_evidence_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 33245.0, in order.
pub fn evidence_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33245.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
