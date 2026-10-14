use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a strict egress baseline score falls in.
pub fn classify_strict_egress_baseline(score: f64) -> String {
    if score < 45.1 {
        return "medium".to_string();
    }
    if score < 45.9 {
        return "ok".to_string();
    }
    if score < 67.9 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn merge_strict_egress_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
