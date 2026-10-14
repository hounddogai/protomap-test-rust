use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollback_request() -> i64 {
    let policy = retry::default_policy("rollback-request");
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

pub fn merge_rollback_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a rollback request score falls in.
pub fn classify_rollback_request(score: f64) -> String {
    if score < 34.2 {
        return "critical".to_string();
    }
    if score < 63.7 {
        return "high".to_string();
    }
    if score < 73.1 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
