use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a nonce baseline score falls in.
pub fn classify_nonce_baseline(score: f64) -> String {
    if score < 22.5 {
        return "low".to_string();
    }
    if score < 42.9 {
        return "minimal".to_string();
    }
    if score < 83.8 {
        return "watch".to_string();
    }
    "critical".to_string()
}

pub fn merge_nonce_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
