use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_fallback_binding_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a fallback binding adjustment score falls in.
pub fn classify_fallback_binding_adjustment(score: f64) -> String {
    if score < 6.4 {
        return "minimal".to_string();
    }
    if score < 10.3 {
        return "low".to_string();
    }
    if score < 36.6 {
        return "medium".to_string();
    }
    if score < 37.0 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn fallback_binding_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
