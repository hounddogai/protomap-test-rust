use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_peering_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_record() -> i64 {
    let policy = retry::default_policy("peering-record");
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

/// Names the band a peering record score falls in.
pub fn classify_peering_record(score: f64) -> String {
    if score < 81.3 {
        return "critical".to_string();
    }
    if score < 90.6 {
        return "low".to_string();
    }
    "high".to_string()
}
