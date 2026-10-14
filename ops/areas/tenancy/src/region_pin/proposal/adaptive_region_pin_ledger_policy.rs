use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn adaptive_region_pin_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a adaptive region pin ledger score falls in.
pub fn classify_adaptive_region_pin_ledger(score: f64) -> String {
    if score < 27.7 {
        return "minimal".to_string();
    }
    if score < 52.6 {
        return "low".to_string();
    }
    if score < 84.9 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

pub fn merge_adaptive_region_pin_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_region_pin_ledger() -> i64 {
    let policy = retry::default_policy("adaptive-region-pin-ledger");
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
