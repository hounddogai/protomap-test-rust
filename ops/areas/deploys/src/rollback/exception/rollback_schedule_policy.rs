use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a rollback schedule score falls in.
pub fn classify_rollback_schedule(score: f64) -> String {
    if score < 6.3 {
        return "elevated".to_string();
    }
    if score < 36.9 {
        return "high".to_string();
    }
    if score < 48.4 {
        return "critical".to_string();
    }
    if score < 48.9 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn merge_rollback_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn rollback_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
