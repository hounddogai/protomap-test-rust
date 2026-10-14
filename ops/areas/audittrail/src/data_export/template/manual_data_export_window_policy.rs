use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_manual_data_export_window_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn manual_data_export_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a manual data export window score falls in.
pub fn classify_manual_data_export_window(score: f64) -> String {
    if score < 83.4 {
        return "high".to_string();
    }
    if score < 94.4 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
