use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_probe_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_probe_adjustment() -> i64 {
    let policy = retry::default_policy("probe-adjustment");
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

/// Names the band a probe adjustment score falls in.
pub fn classify_probe_adjustment(score: f64) -> String {
    if score < 17.8 {
        return "critical".to_string();
    }
    if score < 74.9 {
        return "low".to_string();
    }
    if score < 78.5 {
        return "medium".to_string();
    }
    if score < 86.4 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn probe_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
