use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_batched_runbook_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_runbook_notice() -> i64 {
    let policy = retry::default_policy("batched-runbook-notice");
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

/// Names the band a batched runbook notice score falls in.
pub fn classify_batched_runbook_notice(score: f64) -> String {
    if score < 2.8 {
        return "watch".to_string();
    }
    if score < 86.7 {
        return "ok".to_string();
    }
    if score < 95.2 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
