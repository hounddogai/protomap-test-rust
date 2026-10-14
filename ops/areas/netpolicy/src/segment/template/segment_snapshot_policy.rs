use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_segment_snapshot() -> i64 {
    let policy = retry::default_policy("segment-snapshot");
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

pub fn merge_segment_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a segment snapshot code in its canonical SS- form.
pub fn normalize_segment_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SS-") {
        return clean.clone();
    }
    format!("SS-{}", clean.replace(" ", "-"))
}

/// Names the band a segment snapshot score falls in.
pub fn classify_segment_snapshot(score: f64) -> String {
    if score < 3.3 {
        return "critical".to_string();
    }
    if score < 6.7 {
        return "elevated".to_string();
    }
    if score < 31.9 {
        return "low".to_string();
    }
    if score < 76.8 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
