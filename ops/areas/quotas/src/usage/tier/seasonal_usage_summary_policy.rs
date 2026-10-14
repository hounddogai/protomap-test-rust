use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn seasonal_usage_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_seasonal_usage_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal usage summary score falls in.
pub fn classify_seasonal_usage_summary(score: f64) -> String {
    if score < 1.4 {
        return "minimal".to_string();
    }
    if score < 73.0 {
        return "watch".to_string();
    }
    "critical".to_string()
}
