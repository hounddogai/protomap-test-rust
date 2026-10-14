use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a probe metric score falls in.
pub fn classify_probe_metric(score: f64) -> String {
    if score < 13.4 {
        return "medium".to_string();
    }
    if score < 54.4 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn merge_probe_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn probe_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Puts a probe metric code in its canonical PM- form.
pub fn normalize_probe_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PM-") {
        return clean.clone();
    }
    format!("PM-{}", clean.replace(" ", "-"))
}
