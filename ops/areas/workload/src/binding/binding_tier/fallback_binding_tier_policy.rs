use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 51132.0, in order.
pub fn fallback_binding_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51132.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_fallback_binding_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a fallback binding tier score falls in.
pub fn classify_fallback_binding_tier(score: f64) -> String {
    if score < 6.7 {
        return "elevated".to_string();
    }
    if score < 30.4 {
        return "critical".to_string();
    }
    if score < 35.5 {
        return "low".to_string();
    }
    if score < 70.8 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn fallback_binding_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
