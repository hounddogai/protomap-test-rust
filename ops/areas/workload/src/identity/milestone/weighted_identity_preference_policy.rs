use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn weighted_identity_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a weighted identity preference score falls in.
pub fn classify_weighted_identity_preference(score: f64) -> String {
    if score < 23.1 {
        return "critical".to_string();
    }
    if score < 30.9 {
        return "low".to_string();
    }
    if score < 52.4 {
        return "watch".to_string();
    }
    if score < 71.9 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn merge_weighted_identity_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn weighted_identity_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
