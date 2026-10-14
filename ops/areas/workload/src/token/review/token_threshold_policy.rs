use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_token_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a token threshold score falls in.
pub fn classify_token_threshold(score: f64) -> String {
    if score < 55.3 {
        return "medium".to_string();
    }
    if score < 59.8 {
        return "minimal".to_string();
    }
    if score < 66.9 {
        return "watch".to_string();
    }
    if score < 85.8 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
