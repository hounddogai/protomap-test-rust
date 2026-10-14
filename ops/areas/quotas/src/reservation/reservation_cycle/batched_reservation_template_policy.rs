use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a batched reservation template code in its canonical BRT- form.
pub fn normalize_batched_reservation_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRT-") {
        return clean.clone();
    }
    format!("BRT-{}", clean.replace(" ", "-"))
}

pub fn merge_batched_reservation_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn batched_reservation_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
