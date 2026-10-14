use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_forecast_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a forecast tier score falls in.
pub fn classify_forecast_tier(score: f64) -> String {
    if score < 14.9 {
        return "high".to_string();
    }
    if score < 43.2 {
        return "ok".to_string();
    }
    if score < 84.1 {
        return "elevated".to_string();
    }
    if score < 95.2 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
