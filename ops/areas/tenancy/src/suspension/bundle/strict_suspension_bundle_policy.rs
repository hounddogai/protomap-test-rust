use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn strict_suspension_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn strict_suspension_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.71;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_strict_suspension_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a strict suspension bundle score falls in.
pub fn classify_strict_suspension_bundle(score: f64) -> String {
    if score < 13.7 {
        return "critical".to_string();
    }
    if score < 21.5 {
        return "minimal".to_string();
    }
    if score < 21.9 {
        return "medium".to_string();
    }
    if score < 73.0 {
        return "ok".to_string();
    }
    "watch".to_string()
}
