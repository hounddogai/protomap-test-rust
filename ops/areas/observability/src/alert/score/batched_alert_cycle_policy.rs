use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_batched_alert_cycle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a batched alert cycle score falls in.
pub fn classify_batched_alert_cycle(score: f64) -> String {
    if score < 42.1 {
        return "watch".to_string();
    }
    if score < 45.4 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_alert_cycle() -> i64 {
    let policy = retry::default_policy("batched-alert-cycle");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn batched_alert_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
