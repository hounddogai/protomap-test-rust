use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn fallback_retention_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_retention_schedule() -> i64 {
    let policy = retry::default_policy("fallback-retention-schedule");
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

pub fn fallback_retention_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_fallback_retention_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
