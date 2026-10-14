use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Puts a legacy probe adjustment code in its canonical LPA- form.
pub fn normalize_legacy_probe_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LPA-") {
        return clean.clone();
    }
    format!("LPA-{}", clean.replace(" ", "-"))
}

pub fn legacy_probe_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_probe_adjustment() -> i64 {
    let policy = retry::default_policy("legacy-probe-adjustment");
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

pub fn merge_legacy_probe_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
