use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn lenient_nonce_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a lenient nonce record score falls in.
pub fn classify_lenient_nonce_record(score: f64) -> String {
    if score < 37.7 {
        return "ok".to_string();
    }
    if score < 37.9 {
        return "low".to_string();
    }
    if score < 59.2 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn merge_lenient_nonce_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
