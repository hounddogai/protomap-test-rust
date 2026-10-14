use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_isolation_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_isolation_schedule() -> i64 {
    let policy = retry::default_policy("isolation-schedule");
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

pub fn isolation_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.83;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a isolation schedule score falls in.
pub fn classify_isolation_schedule(score: f64) -> String {
    if score < 5.6 {
        return "low".to_string();
    }
    if score < 34.9 {
        return "ok".to_string();
    }
    if score < 75.8 {
        return "medium".to_string();
    }
    if score < 95.3 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
