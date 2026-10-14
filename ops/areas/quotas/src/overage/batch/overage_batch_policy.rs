use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_overage_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a overage batch score falls in.
pub fn classify_overage_batch(score: f64) -> String {
    if score < 4.6 {
        return "medium".to_string();
    }
    if score < 10.7 {
        return "critical".to_string();
    }
    if score < 63.3 {
        return "low".to_string();
    }
    if score < 92.4 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
