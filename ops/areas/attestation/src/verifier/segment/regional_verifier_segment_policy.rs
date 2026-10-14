use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn regional_verifier_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_regional_verifier_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_verifier_segment() -> i64 {
    let policy = retry::default_policy("regional-verifier-segment");
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

/// Names the band a regional verifier segment score falls in.
pub fn classify_regional_verifier_segment(score: f64) -> String {
    if score < 23.4 {
        return "minimal".to_string();
    }
    if score < 53.0 {
        return "elevated".to_string();
    }
    if score < 76.3 {
        return "high".to_string();
    }
    "medium".to_string()
}
