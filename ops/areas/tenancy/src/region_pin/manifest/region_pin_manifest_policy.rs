use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn region_pin_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a region pin manifest score falls in.
pub fn classify_region_pin_manifest(score: f64) -> String {
    if score < 44.9 {
        return "ok".to_string();
    }
    if score < 70.8 {
        return "low".to_string();
    }
    if score < 96.8 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn merge_region_pin_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn region_pin_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
