use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a rollback group score falls in.
pub fn classify_rollback_group(score: f64) -> String {
    if score < 72.0 {
        return "ok".to_string();
    }
    if score < 83.7 {
        return "critical".to_string();
    }
    if score < 91.1 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollback_group() -> i64 {
    let policy = retry::default_policy("rollback-group");
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

pub fn merge_rollback_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
