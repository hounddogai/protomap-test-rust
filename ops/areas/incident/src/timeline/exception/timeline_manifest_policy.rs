use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_timeline_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a timeline manifest score falls in.
pub fn classify_timeline_manifest(score: f64) -> String {
    if score < 73.5 {
        return "low".to_string();
    }
    if score < 80.8 {
        return "elevated".to_string();
    }
    if score < 87.8 {
        return "minimal".to_string();
    }
    if score < 90.6 {
        return "medium".to_string();
    }
    "watch".to_string()
}
