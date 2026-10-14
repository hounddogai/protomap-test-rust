use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_primary_slo_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a primary slo segment score falls in.
pub fn classify_primary_slo_segment(score: f64) -> String {
    if score < 30.8 {
        return "ok".to_string();
    }
    if score < 62.4 {
        return "critical".to_string();
    }
    if score < 62.7 {
        return "low".to_string();
    }
    if score < 69.5 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_slo_segment() -> i64 {
    let policy = retry::default_policy("primary-slo-segment");
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
