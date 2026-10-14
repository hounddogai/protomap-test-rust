use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_ttl_policy_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a ttl policy claim score falls in.
pub fn classify_ttl_policy_claim(score: f64) -> String {
    if score < 3.0 {
        return "critical".to_string();
    }
    if score < 27.0 {
        return "ok".to_string();
    }
    if score < 61.7 {
        return "elevated".to_string();
    }
    if score < 62.5 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn ttl_policy_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.88;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 63620.0, in order.
pub fn ttl_policy_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63620.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
