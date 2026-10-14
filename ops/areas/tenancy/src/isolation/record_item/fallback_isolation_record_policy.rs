use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_fallback_isolation_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a fallback isolation record score falls in.
pub fn classify_fallback_isolation_record(score: f64) -> String {
    if score < 5.8 {
        return "watch".to_string();
    }
    if score < 20.1 {
        return "minimal".to_string();
    }
    if score < 46.7 {
        return "ok".to_string();
    }
    if score < 64.5 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn fallback_isolation_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
