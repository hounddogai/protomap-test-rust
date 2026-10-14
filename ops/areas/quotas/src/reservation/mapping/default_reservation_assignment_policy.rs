use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_default_reservation_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a default reservation assignment score falls in.
pub fn classify_default_reservation_assignment(score: f64) -> String {
    if score < 6.6 {
        return "elevated".to_string();
    }
    if score < 7.9 {
        return "minimal".to_string();
    }
    if score < 45.3 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn default_reservation_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.9;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
