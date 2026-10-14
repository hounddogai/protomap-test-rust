use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_nonce_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a nonce estimate score falls in.
pub fn classify_nonce_estimate(score: f64) -> String {
    if score < 12.5 {
        return "high".to_string();
    }
    if score < 25.4 {
        return "low".to_string();
    }
    if score < 29.6 {
        return "elevated".to_string();
    }
    if score < 29.6 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
