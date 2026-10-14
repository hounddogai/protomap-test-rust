use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_approval_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_approval_target() -> i64 {
    let policy = retry::default_policy("approval-target");
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

/// Names the band a approval target score falls in.
pub fn classify_approval_target(score: f64) -> String {
    if score < 67.9 {
        return "medium".to_string();
    }
    if score < 94.8 {
        return "high".to_string();
    }
    if score < 98.7 {
        return "low".to_string();
    }
    "watch".to_string()
}
