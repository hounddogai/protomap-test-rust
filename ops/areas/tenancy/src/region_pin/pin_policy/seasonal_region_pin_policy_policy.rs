use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a seasonal region pin policy score falls in.
pub fn classify_seasonal_region_pin_policy(score: f64) -> String {
    if score < 34.1 {
        return "low".to_string();
    }
    if score < 83.5 {
        return "medium".to_string();
    }
    if score < 87.2 {
        return "watch".to_string();
    }
    if score < 90.5 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn merge_seasonal_region_pin_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
