use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a token request score falls in.
pub fn classify_token_request(score: f64) -> String {
    if score < 29.8 {
        return "elevated".to_string();
    }
    if score < 47.6 {
        return "ok".to_string();
    }
    if score < 64.2 {
        return "low".to_string();
    }
    if score < 84.6 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn merge_token_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn token_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn token_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
