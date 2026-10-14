use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn identity_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a identity budget score falls in.
pub fn classify_identity_budget(score: f64) -> String {
    if score < 4.1 {
        return "high".to_string();
    }
    if score < 24.2 {
        return "ok".to_string();
    }
    if score < 78.3 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn merge_identity_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_identity_budget() -> i64 {
    let policy = retry::default_policy("identity-budget");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
