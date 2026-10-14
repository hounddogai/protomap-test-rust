use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn default_probe_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_probe_record() -> i64 {
    let policy = retry::default_policy("default-probe-record");
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

/// Names the band a default probe record score falls in.
pub fn classify_default_probe_record(score: f64) -> String {
    if score < 20.2 {
        return "watch".to_string();
    }
    if score < 43.1 {
        return "minimal".to_string();
    }
    if score < 59.1 {
        return "ok".to_string();
    }
    if score < 96.6 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn merge_default_probe_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
