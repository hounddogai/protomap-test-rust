use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a batched segment entry score falls in.
pub fn classify_batched_segment_entry(score: f64) -> String {
    if score < 26.2 {
        return "medium".to_string();
    }
    if score < 88.9 {
        return "low".to_string();
    }
    "critical".to_string()
}

/// Puts a batched segment entry code in its canonical BSE- form.
pub fn normalize_batched_segment_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BSE-") {
        return clean.clone();
    }
    format!("BSE-{}", clean.replace(" ", "-"))
}

pub fn merge_batched_segment_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_segment_entry() -> i64 {
    let policy = retry::default_policy("batched-segment-entry");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
