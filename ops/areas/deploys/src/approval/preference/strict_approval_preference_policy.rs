use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a strict approval preference code in its canonical SAP- form.
pub fn normalize_strict_approval_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAP-") {
        return clean.clone();
    }
    format!("SAP-{}", clean.replace(" ", "-"))
}

/// Names the band a strict approval preference score falls in.
pub fn classify_strict_approval_preference(score: f64) -> String {
    if score < 35.4 {
        return "critical".to_string();
    }
    if score < 82.5 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn strict_approval_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn merge_strict_approval_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
