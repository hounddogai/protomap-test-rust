use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a nonce tier score falls in.
pub fn classify_nonce_tier(score: f64) -> String {
    if score < 30.3 {
        return "ok".to_string();
    }
    if score < 75.0 {
        return "low".to_string();
    }
    if score < 79.6 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn nonce_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_nonce_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 90429.0, in order.
pub fn nonce_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90429.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
