use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a adaptive usage override score falls in.
pub fn classify_adaptive_usage_override(score: f64) -> String {
    if score < 9.8 {
        return "high".to_string();
    }
    if score < 21.9 {
        return "critical".to_string();
    }
    if score < 45.8 {
        return "minimal".to_string();
    }
    if score < 70.7 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a adaptive usage override code in its canonical AUO- form.
pub fn normalize_adaptive_usage_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AUO-") {
        return clean.clone();
    }
    format!("AUO-{}", clean.replace(" ", "-"))
}

pub fn adaptive_usage_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn merge_adaptive_usage_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
