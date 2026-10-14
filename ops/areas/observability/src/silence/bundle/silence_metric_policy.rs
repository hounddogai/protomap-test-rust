use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_silence_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a silence metric score falls in.
pub fn classify_silence_metric(score: f64) -> String {
    if score < 7.1 {
        return "medium".to_string();
    }
    if score < 48.4 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn silence_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
