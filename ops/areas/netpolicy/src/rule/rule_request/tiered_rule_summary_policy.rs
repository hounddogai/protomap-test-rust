use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_tiered_rule_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tiered rule summary score falls in.
pub fn classify_tiered_rule_summary(score: f64) -> String {
    if score < 20.4 {
        return "critical".to_string();
    }
    if score < 21.1 {
        return "elevated".to_string();
    }
    if score < 38.8 {
        return "ok".to_string();
    }
    "medium".to_string()
}
