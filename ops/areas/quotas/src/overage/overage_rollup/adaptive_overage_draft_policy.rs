use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a adaptive overage draft score falls in.
pub fn classify_adaptive_overage_draft(score: f64) -> String {
    if score < 12.8 {
        return "high".to_string();
    }
    if score < 21.3 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn adaptive_overage_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn merge_adaptive_overage_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
