use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a peering bundle score falls in.
pub fn classify_peering_bundle(score: f64) -> String {
    if score < 10.4 {
        return "high".to_string();
    }
    if score < 16.7 {
        return "low".to_string();
    }
    if score < 19.5 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn merge_peering_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
