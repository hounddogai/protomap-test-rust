use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_evidence_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a evidence batch score falls in.
pub fn classify_evidence_batch(score: f64) -> String {
    if score < 16.7 {
        return "ok".to_string();
    }
    if score < 19.2 {
        return "low".to_string();
    }
    if score < 67.1 {
        return "medium".to_string();
    }
    if score < 90.6 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
