use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a isolation draft score falls in.
pub fn classify_isolation_draft(score: f64) -> String {
    if score < 21.2 {
        return "ok".to_string();
    }
    if score < 36.4 {
        return "medium".to_string();
    }
    if score < 52.4 {
        return "watch".to_string();
    }
    if score < 91.8 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn isolation_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_isolation_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 13138.0, in order.
pub fn isolation_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13138.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
