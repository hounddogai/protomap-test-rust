use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_default_postmortem_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a default postmortem bundle score falls in.
pub fn classify_default_postmortem_bundle(score: f64) -> String {
    if score < 8.3 {
        return "elevated".to_string();
    }
    if score < 23.0 {
        return "watch".to_string();
    }
    if score < 56.8 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn default_postmortem_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn default_postmortem_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
