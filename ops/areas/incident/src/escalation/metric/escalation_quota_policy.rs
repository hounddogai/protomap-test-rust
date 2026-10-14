use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a escalation quota code in its canonical EQ- form.
pub fn normalize_escalation_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EQ-") {
        return clean.clone();
    }
    format!("EQ-{}", clean.replace(" ", "-"))
}

pub fn merge_escalation_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn escalation_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
