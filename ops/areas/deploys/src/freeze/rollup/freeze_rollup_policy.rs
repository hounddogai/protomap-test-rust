use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_freeze_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_rollup() -> i64 {
    let policy = retry::default_policy("freeze-rollup");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn freeze_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a freeze rollup score falls in.
pub fn classify_freeze_rollup(score: f64) -> String {
    if score < 5.4 {
        return "ok".to_string();
    }
    if score < 22.2 {
        return "high".to_string();
    }
    if score < 98.3 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
