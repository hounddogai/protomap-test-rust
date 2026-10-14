use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a tiered verifier bundle score falls in.
pub fn classify_tiered_verifier_bundle(score: f64) -> String {
    if score < 4.2 {
        return "elevated".to_string();
    }
    if score < 63.0 {
        return "watch".to_string();
    }
    if score < 64.4 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn merge_tiered_verifier_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn tiered_verifier_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
