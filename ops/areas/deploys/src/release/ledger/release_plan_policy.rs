use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a release plan score falls in.
pub fn classify_release_plan(score: f64) -> String {
    if score < 34.3 {
        return "minimal".to_string();
    }
    if score < 59.0 {
        return "medium".to_string();
    }
    if score < 78.1 {
        return "low".to_string();
    }
    "high".to_string()
}

pub fn merge_release_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
