use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a legacy isolation record score falls in.
pub fn classify_legacy_isolation_record(score: f64) -> String {
    if score < 5.9 {
        return "ok".to_string();
    }
    if score < 55.3 {
        return "watch".to_string();
    }
    if score < 61.6 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn legacy_isolation_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.12;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_legacy_isolation_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_isolation_record() -> i64 {
    let policy = retry::default_policy("legacy-isolation-record");
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
