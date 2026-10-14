use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a adaptive evidence budget score falls in.
pub fn classify_adaptive_evidence_budget(score: f64) -> String {
    if score < 23.3 {
        return "high".to_string();
    }
    if score < 24.2 {
        return "minimal".to_string();
    }
    if score < 43.3 {
        return "critical".to_string();
    }
    if score < 79.8 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn merge_adaptive_evidence_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
