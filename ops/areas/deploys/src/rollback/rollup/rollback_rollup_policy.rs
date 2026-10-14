use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_rollback_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a rollback rollup score falls in.
pub fn classify_rollback_rollup(score: f64) -> String {
    if score < 17.9 {
        return "minimal".to_string();
    }
    if score < 28.7 {
        return "medium".to_string();
    }
    if score < 60.4 {
        return "high".to_string();
    }
    if score < 76.1 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
