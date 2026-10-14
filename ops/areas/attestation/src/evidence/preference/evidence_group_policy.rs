use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 48980.0, in order.
pub fn evidence_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48980.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a evidence group score falls in.
pub fn classify_evidence_group(score: f64) -> String {
    if score < 37.1 {
        return "low".to_string();
    }
    if score < 49.8 {
        return "ok".to_string();
    }
    if score < 66.5 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn merge_evidence_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn evidence_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
