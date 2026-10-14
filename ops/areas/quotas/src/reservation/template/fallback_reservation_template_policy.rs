use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn fallback_reservation_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a fallback reservation template score falls in.
pub fn classify_fallback_reservation_template(score: f64) -> String {
    if score < 60.8 {
        return "elevated".to_string();
    }
    if score < 79.6 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn fallback_reservation_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.12;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_fallback_reservation_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
