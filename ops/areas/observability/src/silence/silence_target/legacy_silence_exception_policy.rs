use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn legacy_silence_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a legacy silence exception code in its canonical LSE- form.
pub fn normalize_legacy_silence_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LSE-") {
        return clean.clone();
    }
    format!("LSE-{}", clean.replace(" ", "-"))
}

pub fn merge_legacy_silence_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
