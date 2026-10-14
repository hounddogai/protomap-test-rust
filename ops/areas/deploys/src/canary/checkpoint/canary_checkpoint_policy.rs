use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a canary checkpoint score falls in.
pub fn classify_canary_checkpoint(score: f64) -> String {
    if score < 2.0 {
        return "watch".to_string();
    }
    if score < 34.1 {
        return "low".to_string();
    }
    if score < 53.9 {
        return "elevated".to_string();
    }
    if score < 97.3 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_canary_checkpoint() -> i64 {
    let policy = retry::default_policy("canary-checkpoint");
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

pub fn merge_canary_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
