use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_weighted_segment_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a weighted segment batch score falls in.
pub fn classify_weighted_segment_batch(score: f64) -> String {
    if score < 45.8 {
        return "ok".to_string();
    }
    if score < 85.9 {
        return "low".to_string();
    }
    "critical".to_string()
}
