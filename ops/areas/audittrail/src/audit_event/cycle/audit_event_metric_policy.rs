use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a audit event metric score falls in.
pub fn classify_audit_event_metric(score: f64) -> String {
    if score < 45.4 {
        return "elevated".to_string();
    }
    if score < 50.6 {
        return "high".to_string();
    }
    if score < 77.0 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn merge_audit_event_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_audit_event_metric() -> i64 {
    let policy = retry::default_policy("audit-event-metric");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 83191.0, in order.
pub fn audit_event_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83191.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
