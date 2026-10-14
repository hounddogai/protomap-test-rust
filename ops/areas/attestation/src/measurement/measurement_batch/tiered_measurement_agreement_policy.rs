use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a tiered measurement agreement score falls in.
pub fn classify_tiered_measurement_agreement(score: f64) -> String {
    if score < 39.4 {
        return "high".to_string();
    }
    if score < 87.6 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn tiered_measurement_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.97;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_tiered_measurement_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
