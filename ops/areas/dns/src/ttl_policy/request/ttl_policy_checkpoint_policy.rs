use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_ttl_policy_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a ttl policy checkpoint score falls in.
pub fn classify_ttl_policy_checkpoint(score: f64) -> String {
    if score < 24.0 {
        return "high".to_string();
    }
    if score < 26.1 {
        return "watch".to_string();
    }
    if score < 27.9 {
        return "medium".to_string();
    }
    if score < 62.0 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
