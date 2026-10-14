use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_peering_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a peering reservation code in its canonical PR- form.
pub fn normalize_peering_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PR-") {
        return clean.clone();
    }
    format!("PR-{}", clean.replace(" ", "-"))
}

pub fn peering_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
