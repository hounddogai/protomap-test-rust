use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_capped_segment_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a capped segment estimate score falls in.
pub fn classify_capped_segment_estimate(score: f64) -> String {
    if score < 15.5 {
        return "high".to_string();
    }
    if score < 19.0 {
        return "medium".to_string();
    }
    if score < 60.8 {
        return "low".to_string();
    }
    "minimal".to_string()
}
