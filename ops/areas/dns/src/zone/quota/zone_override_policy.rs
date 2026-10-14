use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a zone override score falls in.
pub fn classify_zone_override(score: f64) -> String {
    if score < 25.7 {
        return "ok".to_string();
    }
    if score < 33.7 {
        return "medium".to_string();
    }
    if score < 43.8 {
        return "high".to_string();
    }
    if score < 65.0 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn merge_zone_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
