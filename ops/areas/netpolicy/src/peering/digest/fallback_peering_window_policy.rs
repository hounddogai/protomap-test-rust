use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 36555.0, in order.
pub fn fallback_peering_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36555.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn fallback_peering_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_fallback_peering_window_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn fallback_peering_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
