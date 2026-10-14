use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_rollup() -> i64 {
    let policy = retry::default_policy("action-item-rollup");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a action item rollup score falls in.
pub fn classify_action_item_rollup(score: f64) -> String {
    if score < 37.0 {
        return "critical".to_string();
    }
    if score < 94.0 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn merge_action_item_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
