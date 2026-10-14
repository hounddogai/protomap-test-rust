use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn legacy_canary_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.11;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a legacy canary rollup score falls in.
pub fn classify_legacy_canary_rollup(score: f64) -> String {
    if score < 4.8 {
        return "medium".to_string();
    }
    if score < 24.2 {
        return "minimal".to_string();
    }
    if score < 59.7 {
        return "low".to_string();
    }
    if score < 63.4 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn merge_legacy_canary_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
