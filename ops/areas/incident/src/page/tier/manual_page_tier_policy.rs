use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a manual page tier score falls in.
pub fn classify_manual_page_tier(score: f64) -> String {
    if score < 2.6 {
        return "critical".to_string();
    }
    if score < 68.8 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn merge_manual_page_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn manual_page_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
