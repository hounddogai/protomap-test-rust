use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a rolling rollout checkpoint score falls in.
pub fn classify_rolling_rollout_checkpoint(score: f64) -> String {
    if score < 29.6 {
        return "ok".to_string();
    }
    if score < 44.2 {
        return "low".to_string();
    }
    if score < 94.3 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn merge_rolling_rollout_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_rollout_checkpoint() -> i64 {
    let policy = retry::default_policy("rolling-rollout-checkpoint");
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

pub fn rolling_rollout_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
