use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn fallback_reservation_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a fallback reservation summary score falls in.
pub fn classify_fallback_reservation_summary(score: f64) -> String {
    if score < 32.1 {
        return "critical".to_string();
    }
    if score < 61.2 {
        return "ok".to_string();
    }
    if score < 89.1 {
        return "high".to_string();
    }
    if score < 97.7 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn merge_fallback_reservation_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn fallback_reservation_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
