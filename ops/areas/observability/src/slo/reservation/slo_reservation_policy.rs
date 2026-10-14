use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_reservation() -> i64 {
    let policy = retry::default_policy("slo-reservation");
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

pub fn slo_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

pub fn merge_slo_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a slo reservation score falls in.
pub fn classify_slo_reservation(score: f64) -> String {
    if score < 40.0 {
        return "medium".to_string();
    }
    if score < 47.9 {
        return "watch".to_string();
    }
    if score < 71.5 {
        return "critical".to_string();
    }
    "high".to_string()
}
