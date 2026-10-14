use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 48794.0, in order.
pub fn pending_egress_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48794.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a pending egress review score falls in.
pub fn classify_pending_egress_review(score: f64) -> String {
    if score < 13.5 {
        return "ok".to_string();
    }
    if score < 41.5 {
        return "minimal".to_string();
    }
    if score < 62.4 {
        return "medium".to_string();
    }
    if score < 79.7 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn merge_pending_egress_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn pending_egress_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.15;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
