use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a grant bundle score falls in.
pub fn classify_grant_bundle(score: f64) -> String {
    if score < 69.3 {
        return "minimal".to_string();
    }
    if score < 81.9 {
        return "ok".to_string();
    }
    if score < 88.8 {
        return "high".to_string();
    }
    if score < 91.8 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn merge_grant_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
