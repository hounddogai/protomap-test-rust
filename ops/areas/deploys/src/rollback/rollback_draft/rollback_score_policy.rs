use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a rollback score score falls in.
pub fn classify_rollback_score(score: f64) -> String {
    if score < 36.1 {
        return "high".to_string();
    }
    if score < 43.9 {
        return "minimal".to_string();
    }
    if score < 54.3 {
        return "elevated".to_string();
    }
    if score < 58.1 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn merge_rollback_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn rollback_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn rollback_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
