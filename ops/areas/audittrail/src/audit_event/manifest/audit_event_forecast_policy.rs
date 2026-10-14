use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a audit event forecast score falls in.
pub fn classify_audit_event_forecast(score: f64) -> String {
    if score < 2.4 {
        return "watch".to_string();
    }
    if score < 7.2 {
        return "minimal".to_string();
    }
    if score < 80.1 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 36263.0, in order.
pub fn audit_event_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36263.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn audit_event_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

pub fn merge_audit_event_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
