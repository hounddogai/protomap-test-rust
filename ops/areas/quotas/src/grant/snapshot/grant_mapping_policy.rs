use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn grant_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_grant_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a grant mapping score falls in.
pub fn classify_grant_mapping(score: f64) -> String {
    if score < 33.9 {
        return "low".to_string();
    }
    if score < 84.1 {
        return "high".to_string();
    }
    "medium".to_string()
}
