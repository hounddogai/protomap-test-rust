use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn peering_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_agreement() -> i64 {
    let policy = retry::default_policy("peering-agreement");
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

/// Names the band a peering agreement score falls in.
pub fn classify_peering_agreement(score: f64) -> String {
    if score < 30.8 {
        return "minimal".to_string();
    }
    if score < 77.1 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn merge_peering_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
