use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 2636.0, in order.
pub fn regional_audit_event_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2636.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_regional_audit_event_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn regional_audit_event_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a regional audit event snapshot score falls in.
pub fn classify_regional_audit_event_snapshot(score: f64) -> String {
    if score < 12.3 {
        return "high".to_string();
    }
    if score < 61.7 {
        return "low".to_string();
    }
    if score < 75.2 {
        return "minimal".to_string();
    }
    if score < 83.3 {
        return "watch".to_string();
    }
    "critical".to_string()
}
