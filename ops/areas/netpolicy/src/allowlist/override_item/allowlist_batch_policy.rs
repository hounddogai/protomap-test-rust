use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_allowlist_batch() -> i64 {
    let policy = retry::default_policy("allowlist-batch");
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

/// Names the band a allowlist batch score falls in.
pub fn classify_allowlist_batch(score: f64) -> String {
    if score < 2.6 {
        return "low".to_string();
    }
    if score < 61.1 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Puts a allowlist batch code in its canonical AB- form.
pub fn normalize_allowlist_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AB-") {
        return clean.clone();
    }
    format!("AB-{}", clean.replace(" ", "-"))
}

pub fn merge_allowlist_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
