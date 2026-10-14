use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn measurement_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn measurement_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.19;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a measurement manifest score falls in.
pub fn classify_measurement_manifest(score: f64) -> String {
    if score < 80.4 {
        return "minimal".to_string();
    }
    if score < 87.7 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn merge_measurement_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
