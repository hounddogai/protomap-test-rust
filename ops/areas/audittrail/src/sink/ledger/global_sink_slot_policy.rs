use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a global sink slot score falls in.
pub fn classify_global_sink_slot(score: f64) -> String {
    if score < 11.2 {
        return "low".to_string();
    }
    if score < 26.2 {
        return "elevated".to_string();
    }
    if score < 64.1 {
        return "high".to_string();
    }
    if score < 73.3 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn merge_global_sink_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_sink_slot() -> i64 {
    let policy = retry::default_policy("global-sink-slot");
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
