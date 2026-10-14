use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_tiered_segment_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tiered segment forecast score falls in.
pub fn classify_tiered_segment_forecast(score: f64) -> String {
    if score < 27.7 {
        return "low".to_string();
    }
    if score < 42.6 {
        return "elevated".to_string();
    }
    if score < 51.5 {
        return "critical".to_string();
    }
    "high".to_string()
}
