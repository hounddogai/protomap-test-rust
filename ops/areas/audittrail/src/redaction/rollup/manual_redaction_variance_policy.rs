use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_manual_redaction_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a manual redaction variance score falls in.
pub fn classify_manual_redaction_variance(score: f64) -> String {
    if score < 4.0 {
        return "medium".to_string();
    }
    if score < 18.8 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
