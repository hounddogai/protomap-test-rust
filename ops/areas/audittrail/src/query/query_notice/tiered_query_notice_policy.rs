use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn tiered_query_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a tiered query notice score falls in.
pub fn classify_tiered_query_notice(score: f64) -> String {
    if score < 29.2 {
        return "watch".to_string();
    }
    if score < 66.0 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn merge_tiered_query_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tiered query notice code in its canonical TQN- form.
pub fn normalize_tiered_query_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TQN-") {
        return clean.clone();
    }
    format!("TQN-{}", clean.replace(" ", "-"))
}
