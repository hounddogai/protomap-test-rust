use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a redaction adjustment score falls in.
pub fn classify_redaction_adjustment(score: f64) -> String {
    if score < 12.2 {
        return "minimal".to_string();
    }
    if score < 36.1 {
        return "elevated".to_string();
    }
    if score < 57.6 {
        return "medium".to_string();
    }
    if score < 61.6 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn merge_redaction_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
