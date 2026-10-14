use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a canary segment score falls in.
pub fn classify_canary_segment(score: f64) -> String {
    if score < 82.8 {
        return "minimal".to_string();
    }
    if score < 89.7 {
        return "medium".to_string();
    }
    if score < 93.2 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn merge_canary_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_canary_segment() -> i64 {
    let policy = retry::default_policy("canary-segment");
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
