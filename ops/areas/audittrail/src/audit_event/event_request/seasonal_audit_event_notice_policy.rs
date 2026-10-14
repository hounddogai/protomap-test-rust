use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a seasonal audit event notice code in its canonical SAEN- form.
pub fn normalize_seasonal_audit_event_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAEN-") {
        return clean.clone();
    }
    format!("SAEN-{}", clean.replace(" ", "-"))
}

pub fn merge_seasonal_audit_event_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn seasonal_audit_event_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
