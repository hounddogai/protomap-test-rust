use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a scheduled isolation exception score falls in.
pub fn classify_scheduled_isolation_exception(score: f64) -> String {
    if score < 17.9 {
        return "critical".to_string();
    }
    if score < 47.0 {
        return "ok".to_string();
    }
    if score < 98.1 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn merge_scheduled_isolation_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_isolation_exception() -> i64 {
    let policy = retry::default_policy("scheduled-isolation-exception");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
