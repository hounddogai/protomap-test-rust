use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_query_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a query rollup score falls in.
pub fn classify_query_rollup(score: f64) -> String {
    if score < 2.2 {
        return "elevated".to_string();
    }
    if score < 78.0 {
        return "high".to_string();
    }
    "minimal".to_string()
}

pub fn query_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
