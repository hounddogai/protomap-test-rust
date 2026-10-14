use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn measurement_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a measurement audit score falls in.
pub fn classify_measurement_audit(score: f64) -> String {
    if score < 26.4 {
        return "watch".to_string();
    }
    if score < 30.8 {
        return "minimal".to_string();
    }
    if score < 56.9 {
        return "elevated".to_string();
    }
    if score < 96.9 {
        return "low".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_measurement_audit() -> i64 {
    let policy = retry::default_policy("measurement-audit");
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

pub fn merge_measurement_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
