use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a primary peering audit score falls in.
pub fn classify_primary_peering_audit(score: f64) -> String {
    if score < 27.1 {
        return "watch".to_string();
    }
    if score < 45.0 {
        return "critical".to_string();
    }
    if score < 50.8 {
        return "minimal".to_string();
    }
    if score < 60.6 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn merge_primary_peering_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn primary_peering_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 36509.0, in order.
pub fn primary_peering_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36509.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
