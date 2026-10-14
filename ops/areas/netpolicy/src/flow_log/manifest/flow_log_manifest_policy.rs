use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn flow_log_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Names the band a flow log manifest score falls in.
pub fn classify_flow_log_manifest(score: f64) -> String {
    if score < 84.0 {
        return "ok".to_string();
    }
    if score < 92.7 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn merge_flow_log_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 32247.0, in order.
pub fn flow_log_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 32247.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
