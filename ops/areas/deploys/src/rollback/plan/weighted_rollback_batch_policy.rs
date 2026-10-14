use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_weighted_rollback_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a weighted rollback batch score falls in.
pub fn classify_weighted_rollback_batch(score: f64) -> String {
    if score < 29.4 {
        return "ok".to_string();
    }
    if score < 72.4 {
        return "high".to_string();
    }
    if score < 84.8 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
