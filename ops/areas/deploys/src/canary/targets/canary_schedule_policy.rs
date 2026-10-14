use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a canary schedule score falls in.
pub fn classify_canary_schedule(score: f64) -> String {
    if score < 5.5 {
        return "low".to_string();
    }
    if score < 37.2 {
        return "ok".to_string();
    }
    if score < 44.9 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn merge_canary_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_canary_schedule() -> i64 {
    let policy = retry::default_policy("canary-schedule");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
