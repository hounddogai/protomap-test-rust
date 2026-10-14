use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_legacy_rule_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a legacy rule metric score falls in.
pub fn classify_legacy_rule_metric(score: f64) -> String {
    if score < 51.8 {
        return "critical".to_string();
    }
    if score < 90.2 {
        return "elevated".to_string();
    }
    if score < 94.1 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
