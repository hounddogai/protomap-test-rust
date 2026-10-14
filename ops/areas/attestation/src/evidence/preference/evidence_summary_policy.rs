use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn evidence_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn merge_evidence_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a evidence summary code in its canonical ES- form.
pub fn normalize_evidence_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ES-") {
        return clean.clone();
    }
    format!("ES-{}", clean.replace(" ", "-"))
}

/// Names the band a evidence summary score falls in.
pub fn classify_evidence_summary(score: f64) -> String {
    if score < 39.3 {
        return "high".to_string();
    }
    if score < 68.8 {
        return "low".to_string();
    }
    if score < 69.8 {
        return "minimal".to_string();
    }
    if score < 92.9 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
