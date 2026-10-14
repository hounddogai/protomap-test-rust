use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn weighted_dashboard_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a weighted dashboard profile score falls in.
pub fn classify_weighted_dashboard_profile(score: f64) -> String {
    if score < 13.2 {
        return "critical".to_string();
    }
    if score < 34.8 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

/// Puts a weighted dashboard profile code in its canonical WDP- form.
pub fn normalize_weighted_dashboard_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WDP-") {
        return clean.clone();
    }
    format!("WDP-{}", clean.replace(" ", "-"))
}

pub fn merge_weighted_dashboard_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
