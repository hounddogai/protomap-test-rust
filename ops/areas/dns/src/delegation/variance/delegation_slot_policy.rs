use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn delegation_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a delegation slot score falls in.
pub fn classify_delegation_slot(score: f64) -> String {
    if score < 18.1 {
        return "critical".to_string();
    }
    if score < 24.5 {
        return "minimal".to_string();
    }
    if score < 40.1 {
        return "ok".to_string();
    }
    if score < 64.0 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn merge_delegation_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
