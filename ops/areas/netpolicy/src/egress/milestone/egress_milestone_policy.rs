use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_egress_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a egress milestone score falls in.
pub fn classify_egress_milestone(score: f64) -> String {
    if score < 46.2 {
        return "watch".to_string();
    }
    if score < 78.1 {
        return "elevated".to_string();
    }
    if score < 81.8 {
        return "ok".to_string();
    }
    "critical".to_string()
}
