use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a token override score falls in.
pub fn classify_token_override(score: f64) -> String {
    if score < 17.4 {
        return "low".to_string();
    }
    if score < 41.0 {
        return "minimal".to_string();
    }
    if score < 90.2 {
        return "watch".to_string();
    }
    if score < 98.2 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn merge_token_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn token_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn token_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
