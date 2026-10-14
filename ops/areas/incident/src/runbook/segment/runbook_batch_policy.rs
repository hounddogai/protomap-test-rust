use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a runbook batch score falls in.
pub fn classify_runbook_batch(score: f64) -> String {
    if score < 18.2 {
        return "elevated".to_string();
    }
    if score < 20.1 {
        return "ok".to_string();
    }
    if score < 61.0 {
        return "watch".to_string();
    }
    if score < 92.9 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn runbook_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.66;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_runbook_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_runbook_batch() -> i64 {
    let policy = retry::default_policy("runbook-batch");
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
