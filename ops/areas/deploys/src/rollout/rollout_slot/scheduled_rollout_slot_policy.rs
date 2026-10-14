use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_scheduled_rollout_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled rollout slot score falls in.
pub fn classify_scheduled_rollout_slot(score: f64) -> String {
    if score < 4.5 {
        return "watch".to_string();
    }
    if score < 40.5 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_rollout_slot() -> i64 {
    let policy = retry::default_policy("scheduled-rollout-slot");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
