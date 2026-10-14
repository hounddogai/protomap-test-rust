use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a batched runbook summary code in its canonical BRS- form.
pub fn normalize_batched_runbook_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRS-") {
        return clean.clone();
    }
    format!("BRS-{}", clean.replace(" ", "-"))
}

pub fn merge_batched_runbook_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn batched_runbook_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a batched runbook summary score falls in.
pub fn classify_batched_runbook_summary(score: f64) -> String {
    if score < 28.1 {
        return "high".to_string();
    }
    if score < 58.6 {
        return "watch".to_string();
    }
    if score < 70.6 {
        return "ok".to_string();
    }
    if score < 85.3 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
