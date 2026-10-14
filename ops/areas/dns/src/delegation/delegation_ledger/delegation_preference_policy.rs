use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_delegation_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a delegation preference score falls in.
pub fn classify_delegation_preference(score: f64) -> String {
    if score < 24.4 {
        return "critical".to_string();
    }
    if score < 62.7 {
        return "watch".to_string();
    }
    "low".to_string()
}
