use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_legacy_audit_event_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn legacy_audit_event_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a legacy audit event schedule score falls in.
pub fn classify_legacy_audit_event_schedule(score: f64) -> String {
    if score < 26.4 {
        return "minimal".to_string();
    }
    if score < 88.8 {
        return "ok".to_string();
    }
    if score < 98.2 {
        return "medium".to_string();
    }
    "high".to_string()
}
