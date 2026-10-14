use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_escalation_rollup() -> i64 {
    let policy = retry::default_policy("default-escalation-rollup");
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

pub fn merge_default_escalation_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn default_escalation_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Names the band a default escalation rollup score falls in.
pub fn classify_default_escalation_rollup(score: f64) -> String {
    if score < 46.1 {
        return "high".to_string();
    }
    if score < 62.1 {
        return "critical".to_string();
    }
    if score < 82.3 {
        return "medium".to_string();
    }
    if score < 91.6 {
        return "ok".to_string();
    }
    "low".to_string()
}
