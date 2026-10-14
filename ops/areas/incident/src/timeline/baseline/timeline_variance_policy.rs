use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn timeline_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_timeline_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a timeline variance score falls in.
pub fn classify_timeline_variance(score: f64) -> String {
    if score < 3.9 {
        return "low".to_string();
    }
    if score < 18.3 {
        return "high".to_string();
    }
    if score < 24.2 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
