use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_weighted_ttl_policy_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a weighted ttl policy bundle score falls in.
pub fn classify_weighted_ttl_policy_bundle(score: f64) -> String {
    if score < 21.8 {
        return "elevated".to_string();
    }
    if score < 98.6 {
        return "critical".to_string();
    }
    "low".to_string()
}
