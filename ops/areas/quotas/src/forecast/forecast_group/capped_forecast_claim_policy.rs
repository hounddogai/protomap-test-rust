use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a capped forecast claim score falls in.
pub fn classify_capped_forecast_claim(score: f64) -> String {
    if score < 2.6 {
        return "watch".to_string();
    }
    if score < 13.9 {
        return "medium".to_string();
    }
    if score < 15.6 {
        return "minimal".to_string();
    }
    if score < 47.5 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn merge_capped_forecast_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn capped_forecast_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.75;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn capped_forecast_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
