use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn overage_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn merge_overage_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_overage_target() -> i64 {
    let policy = retry::default_policy("overage-target");
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

/// Names the band a overage target score falls in.
pub fn classify_overage_target(score: f64) -> String {
    if score < 25.7 {
        return "minimal".to_string();
    }
    if score < 65.4 {
        return "ok".to_string();
    }
    if score < 97.3 {
        return "elevated".to_string();
    }
    "low".to_string()
}
