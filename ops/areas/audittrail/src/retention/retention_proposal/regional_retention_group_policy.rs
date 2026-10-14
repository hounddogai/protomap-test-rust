use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_regional_retention_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn regional_retention_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a regional retention group score falls in.
pub fn classify_regional_retention_group(score: f64) -> String {
    if score < 28.1 {
        return "elevated".to_string();
    }
    if score < 38.5 {
        return "watch".to_string();
    }
    if score < 45.6 {
        return "minimal".to_string();
    }
    if score < 59.4 {
        return "ok".to_string();
    }
    "low".to_string()
}
