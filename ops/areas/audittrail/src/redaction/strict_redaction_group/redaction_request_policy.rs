use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a redaction request score falls in.
pub fn classify_redaction_request(score: f64) -> String {
    if score < 54.6 {
        return "ok".to_string();
    }
    if score < 89.3 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_redaction_request() -> i64 {
    let policy = retry::default_policy("redaction-request");
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

pub fn merge_redaction_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn redaction_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
