use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn usage_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_usage_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn usage_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a usage target score falls in.
pub fn classify_usage_target(score: f64) -> String {
    if score < 27.2 {
        return "critical".to_string();
    }
    if score < 47.5 {
        return "minimal".to_string();
    }
    if score < 48.8 {
        return "ok".to_string();
    }
    if score < 84.1 {
        return "elevated".to_string();
    }
    "high".to_string()
}
