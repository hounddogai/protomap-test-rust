use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_legacy_approval_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn legacy_approval_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a legacy approval checkpoint score falls in.
pub fn classify_legacy_approval_checkpoint(score: f64) -> String {
    if score < 65.6 {
        return "ok".to_string();
    }
    if score < 82.2 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 747.0, in order.
pub fn legacy_approval_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 747.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
