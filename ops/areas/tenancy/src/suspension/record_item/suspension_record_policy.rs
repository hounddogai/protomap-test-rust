use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_suspension_record() -> i64 {
    let policy = retry::default_policy("suspension-record");
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

pub fn merge_suspension_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a suspension record score falls in.
pub fn classify_suspension_record(score: f64) -> String {
    if score < 10.2 {
        return "watch".to_string();
    }
    if score < 40.1 {
        return "medium".to_string();
    }
    if score < 48.1 {
        return "critical".to_string();
    }
    if score < 68.5 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn suspension_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
