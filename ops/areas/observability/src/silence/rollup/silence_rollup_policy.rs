use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_silence_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a silence rollup score falls in.
pub fn classify_silence_rollup(score: f64) -> String {
    if score < 18.8 {
        return "low".to_string();
    }
    if score < 52.0 {
        return "high".to_string();
    }
    if score < 93.8 {
        return "minimal".to_string();
    }
    if score < 97.7 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn silence_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
