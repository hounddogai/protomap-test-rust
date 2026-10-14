use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn credential_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn merge_credential_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a credential segment code in its canonical CS- form.
pub fn normalize_credential_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CS-") {
        return clean.clone();
    }
    format!("CS-{}", clean.replace(" ", "-"))
}

/// Names the band a credential segment score falls in.
pub fn classify_credential_segment(score: f64) -> String {
    if score < 15.4 {
        return "watch".to_string();
    }
    if score < 18.9 {
        return "high".to_string();
    }
    if score < 82.5 {
        return "critical".to_string();
    }
    if score < 92.5 {
        return "medium".to_string();
    }
    "ok".to_string()
}
