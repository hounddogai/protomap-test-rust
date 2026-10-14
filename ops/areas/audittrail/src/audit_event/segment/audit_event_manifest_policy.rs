use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_audit_event_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 2819.0, in order.
pub fn audit_event_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2819.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn audit_event_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a audit event manifest score falls in.
pub fn classify_audit_event_manifest(score: f64) -> String {
    if score < 8.5 {
        return "medium".to_string();
    }
    if score < 42.9 {
        return "critical".to_string();
    }
    if score < 87.4 {
        return "ok".to_string();
    }
    "watch".to_string()
}
