use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a tiered egress digest score falls in.
pub fn classify_tiered_egress_digest(score: f64) -> String {
    if score < 39.6 {
        return "high".to_string();
    }
    if score < 42.6 {
        return "ok".to_string();
    }
    if score < 85.0 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn tiered_egress_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn tiered_egress_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn merge_tiered_egress_digest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
