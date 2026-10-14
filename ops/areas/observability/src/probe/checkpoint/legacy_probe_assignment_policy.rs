use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_legacy_probe_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_probe_assignment() -> i64 {
    let policy = retry::default_policy("legacy-probe-assignment");
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

/// Names the band a legacy probe assignment score falls in.
pub fn classify_legacy_probe_assignment(score: f64) -> String {
    if score < 12.3 {
        return "watch".to_string();
    }
    if score < 18.1 {
        return "medium".to_string();
    }
    if score < 27.0 {
        return "high".to_string();
    }
    if score < 94.6 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
