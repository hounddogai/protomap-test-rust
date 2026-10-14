use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a credential plan score falls in.
pub fn classify_credential_plan(score: f64) -> String {
    if score < 14.0 {
        return "elevated".to_string();
    }
    if score < 43.1 {
        return "ok".to_string();
    }
    if score < 57.7 {
        return "critical".to_string();
    }
    if score < 72.1 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn credential_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn merge_credential_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_credential_plan() -> i64 {
    let policy = retry::default_policy("credential-plan");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
