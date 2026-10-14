use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a regional usage segment code in its canonical RUS- form.
pub fn normalize_regional_usage_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RUS-") {
        return clean.clone();
    }
    format!("RUS-{}", clean.replace(" ", "-"))
}

pub fn merge_regional_usage_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn regional_usage_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
