use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a silence group score falls in.
pub fn classify_silence_group(score: f64) -> String {
    if score < 23.8 {
        return "watch".to_string();
    }
    if score < 89.6 {
        return "critical".to_string();
    }
    if score < 91.5 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn silence_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn merge_silence_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_silence_group() -> i64 {
    let policy = retry::default_policy("silence-group");
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
