use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a lenient usage forecast score falls in.
pub fn classify_lenient_usage_forecast(score: f64) -> String {
    if score < 8.6 {
        return "ok".to_string();
    }
    if score < 42.2 {
        return "medium".to_string();
    }
    if score < 86.3 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn lenient_usage_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_lenient_usage_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
