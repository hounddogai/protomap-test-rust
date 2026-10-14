use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_approval_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a approval rollup score falls in.
pub fn classify_approval_rollup(score: f64) -> String {
    if score < 23.3 {
        return "watch".to_string();
    }
    if score < 77.6 {
        return "ok".to_string();
    }
    "critical".to_string()
}
