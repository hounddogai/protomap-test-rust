use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_tiered_probe_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tiered probe agreement score falls in.
pub fn classify_tiered_probe_agreement(score: f64) -> String {
    if score < 42.5 {
        return "critical".to_string();
    }
    if score < 90.2 {
        return "high".to_string();
    }
    if score < 93.7 {
        return "ok".to_string();
    }
    if score < 99.6 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn tiered_probe_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_probe_agreement() -> i64 {
    let policy = retry::default_policy("tiered-probe-agreement");
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
