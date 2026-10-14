use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 14075.0, in order.
pub fn redaction_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14075.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a redaction segment score falls in.
pub fn classify_redaction_segment(score: f64) -> String {
    if score < 19.5 {
        return "minimal".to_string();
    }
    if score < 32.1 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn merge_redaction_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn redaction_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
