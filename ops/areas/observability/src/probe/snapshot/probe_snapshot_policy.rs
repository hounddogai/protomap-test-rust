use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a probe snapshot score falls in.
pub fn classify_probe_snapshot(score: f64) -> String {
    if score < 9.8 {
        return "minimal".to_string();
    }
    if score < 50.9 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn merge_probe_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
