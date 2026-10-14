use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_audit_event_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a audit event snapshot score falls in.
pub fn classify_audit_event_snapshot(score: f64) -> String {
    if score < 17.0 {
        return "elevated".to_string();
    }
    if score < 24.0 {
        return "minimal".to_string();
    }
    if score < 49.0 {
        return "watch".to_string();
    }
    if score < 55.8 {
        return "high".to_string();
    }
    "ok".to_string()
}

/// Puts a audit event snapshot code in its canonical AES- form.
pub fn normalize_audit_event_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AES-") {
        return clean.clone();
    }
    format!("AES-{}", clean.replace(" ", "-"))
}
