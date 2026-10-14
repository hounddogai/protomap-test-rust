use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a tiered credential plan score falls in.
pub fn classify_tiered_credential_plan(score: f64) -> String {
    if score < 8.0 {
        return "low".to_string();
    }
    if score < 73.0 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn merge_tiered_credential_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
