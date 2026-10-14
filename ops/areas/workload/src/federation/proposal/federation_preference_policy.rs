use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a federation preference code in its canonical FP- form.
pub fn normalize_federation_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FP-") {
        return clean.clone();
    }
    format!("FP-{}", clean.replace(" ", "-"))
}

pub fn federation_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a federation preference score falls in.
pub fn classify_federation_preference(score: f64) -> String {
    if score < 8.4 {
        return "minimal".to_string();
    }
    if score < 45.8 {
        return "medium".to_string();
    }
    if score < 79.3 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn merge_federation_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
