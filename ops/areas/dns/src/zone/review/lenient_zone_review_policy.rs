use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a lenient zone review code in its canonical LZR- form.
pub fn normalize_lenient_zone_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LZR-") {
        return clean.clone();
    }
    format!("LZR-{}", clean.replace(" ", "-"))
}

pub fn lenient_zone_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn merge_lenient_zone_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
