use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_fallback_data_export_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a fallback data export rule score falls in.
pub fn classify_fallback_data_export_rule(score: f64) -> String {
    if score < 18.3 {
        return "elevated".to_string();
    }
    if score < 19.3 {
        return "watch".to_string();
    }
    if score < 34.3 {
        return "low".to_string();
    }
    if score < 45.4 {
        return "medium".to_string();
    }
    "ok".to_string()
}
