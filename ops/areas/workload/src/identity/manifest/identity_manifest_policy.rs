use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_identity_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a identity manifest score falls in.
pub fn classify_identity_manifest(score: f64) -> String {
    if score < 10.3 {
        return "critical".to_string();
    }
    if score < 20.9 {
        return "low".to_string();
    }
    if score < 45.3 {
        return "minimal".to_string();
    }
    if score < 84.5 {
        return "medium".to_string();
    }
    "ok".to_string()
}
