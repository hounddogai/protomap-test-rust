use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn seasonal_peering_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn merge_seasonal_peering_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a seasonal peering group code in its canonical SPG- form.
pub fn normalize_seasonal_peering_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPG-") {
        return clean.clone();
    }
    format!("SPG-{}", clean.replace(" ", "-"))
}
