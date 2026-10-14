use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn grant_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.87;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_grant_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn grant_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a grant threshold score falls in.
pub fn classify_grant_threshold(score: f64) -> String {
    if score < 16.6 {
        return "elevated".to_string();
    }
    if score < 18.3 {
        return "ok".to_string();
    }
    if score < 85.0 {
        return "low".to_string();
    }
    "watch".to_string()
}
