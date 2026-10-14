use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a dns record threshold score falls in.
pub fn classify_dns_record_threshold(score: f64) -> String {
    if score < 8.2 {
        return "watch".to_string();
    }
    if score < 48.9 {
        return "minimal".to_string();
    }
    if score < 86.8 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn dns_record_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dns_record_threshold() -> i64 {
    let policy = retry::default_policy("dns-record-threshold");
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

pub fn merge_dns_record_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
