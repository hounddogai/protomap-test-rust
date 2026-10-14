use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_rollback_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollback_assignment() -> i64 {
    let policy = retry::default_policy("rollback-assignment");
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

pub fn rollback_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rollback assignment score falls in.
pub fn classify_rollback_assignment(score: f64) -> String {
    if score < 11.1 {
        return "ok".to_string();
    }
    if score < 29.1 {
        return "low".to_string();
    }
    if score < 32.6 {
        return "elevated".to_string();
    }
    if score < 65.2 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
