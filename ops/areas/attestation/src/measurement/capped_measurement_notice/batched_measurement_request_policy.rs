use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a batched measurement request score falls in.
pub fn classify_batched_measurement_request(score: f64) -> String {
    if score < 31.7 {
        return "ok".to_string();
    }
    if score < 43.9 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn merge_batched_measurement_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
