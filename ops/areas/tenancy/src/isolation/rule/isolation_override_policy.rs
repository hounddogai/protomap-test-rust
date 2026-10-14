use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a isolation override score falls in.
pub fn classify_isolation_override(score: f64) -> String {
    if score < 73.0 {
        return "elevated".to_string();
    }
    if score < 79.9 {
        return "critical".to_string();
    }
    if score < 89.5 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn isolation_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Puts a isolation override code in its canonical IO- form.
pub fn normalize_isolation_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IO-") {
        return clean.clone();
    }
    format!("IO-{}", clean.replace(" ", "-"))
}

pub fn merge_isolation_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
