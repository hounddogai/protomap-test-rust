use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_pending_region_pin_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a pending region pin proposal score falls in.
pub fn classify_pending_region_pin_proposal(score: f64) -> String {
    if score < 38.8 {
        return "watch".to_string();
    }
    if score < 80.3 {
        return "ok".to_string();
    }
    if score < 86.8 {
        return "elevated".to_string();
    }
    if score < 93.6 {
        return "low".to_string();
    }
    "critical".to_string()
}
