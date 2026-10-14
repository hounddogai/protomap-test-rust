use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_health_check_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn health_check_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a health check threshold code in its canonical HCT- form.
pub fn normalize_health_check_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCT-") {
        return clean.clone();
    }
    format!("HCT-{}", clean.replace(" ", "-"))
}
