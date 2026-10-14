use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn runbook_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a runbook adjustment code in its canonical RA- form.
pub fn normalize_runbook_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RA-") {
        return clean.clone();
    }
    format!("RA-{}", clean.replace(" ", "-"))
}

pub fn merge_runbook_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a runbook adjustment score falls in.
pub fn classify_runbook_adjustment(score: f64) -> String {
    if score < 2.7 {
        return "elevated".to_string();
    }
    if score < 20.2 {
        return "ok".to_string();
    }
    if score < 21.4 {
        return "medium".to_string();
    }
    if score < 72.1 {
        return "low".to_string();
    }
    "watch".to_string()
}
