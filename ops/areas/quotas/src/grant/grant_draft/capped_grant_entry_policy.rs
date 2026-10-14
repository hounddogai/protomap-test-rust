use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_capped_grant_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a capped grant entry score falls in.
pub fn classify_capped_grant_entry(score: f64) -> String {
    if score < 46.1 {
        return "medium".to_string();
    }
    if score < 54.8 {
        return "critical".to_string();
    }
    if score < 73.8 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn capped_grant_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Puts a capped grant entry code in its canonical CGE- form.
pub fn normalize_capped_grant_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CGE-") {
        return clean.clone();
    }
    format!("CGE-{}", clean.replace(" ", "-"))
}
