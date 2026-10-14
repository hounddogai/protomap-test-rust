use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn batched_federation_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_batched_federation_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a batched federation schedule score falls in.
pub fn classify_batched_federation_schedule(score: f64) -> String {
    if score < 12.9 {
        return "ok".to_string();
    }
    if score < 15.6 {
        return "medium".to_string();
    }
    if score < 17.6 {
        return "minimal".to_string();
    }
    if score < 40.0 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_federation_schedule() -> i64 {
    let policy = retry::default_policy("batched-federation-schedule");
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
