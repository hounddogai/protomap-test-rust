use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a fallback page variance score falls in.
pub fn classify_fallback_page_variance(score: f64) -> String {
    if score < 28.0 {
        return "high".to_string();
    }
    if score < 32.1 {
        return "medium".to_string();
    }
    if score < 34.9 {
        return "watch".to_string();
    }
    if score < 86.1 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn merge_fallback_page_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
