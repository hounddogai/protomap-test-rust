use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_delegation_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a delegation proposal score falls in.
pub fn classify_delegation_proposal(score: f64) -> String {
    if score < 51.2 {
        return "elevated".to_string();
    }
    if score < 80.4 {
        return "medium".to_string();
    }
    if score < 81.1 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn delegation_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
