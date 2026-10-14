use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_manual_identity_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn manual_identity_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Puts a manual identity forecast code in its canonical MIF- form.
pub fn normalize_manual_identity_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MIF-") {
        return clean.clone();
    }
    format!("MIF-{}", clean.replace(" ", "-"))
}
