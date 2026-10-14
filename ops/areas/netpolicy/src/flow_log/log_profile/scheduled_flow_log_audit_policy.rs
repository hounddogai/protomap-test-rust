use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn scheduled_flow_log_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Puts a scheduled flow log audit code in its canonical SFLA- form.
pub fn normalize_scheduled_flow_log_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFLA-") {
        return clean.clone();
    }
    format!("SFLA-{}", clean.replace(" ", "-"))
}

pub fn merge_scheduled_flow_log_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
