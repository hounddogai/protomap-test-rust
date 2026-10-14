use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_redaction_draft() -> i64 {
    let policy = retry::default_policy("batched-redaction-draft");
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

pub fn batched_redaction_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn merge_batched_redaction_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a batched redaction draft score falls in.
pub fn classify_batched_redaction_draft(score: f64) -> String {
    if score < 16.7 {
        return "medium".to_string();
    }
    if score < 55.5 {
        return "watch".to_string();
    }
    if score < 79.1 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
