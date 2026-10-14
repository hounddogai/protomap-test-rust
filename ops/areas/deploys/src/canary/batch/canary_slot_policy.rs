use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_canary_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a canary slot score falls in.
pub fn classify_canary_slot(score: f64) -> String {
    if score < 37.9 {
        return "elevated".to_string();
    }
    if score < 77.2 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn canary_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a canary slot code in its canonical CS- form.
pub fn normalize_canary_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CS-") {
        return clean.clone();
    }
    format!("CS-{}", clean.replace(" ", "-"))
}
