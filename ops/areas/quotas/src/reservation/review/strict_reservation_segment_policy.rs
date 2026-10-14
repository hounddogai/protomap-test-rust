use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn strict_reservation_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_reservation_segment() -> i64 {
    let policy = retry::default_policy("strict-reservation-segment");
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

/// Names the band a strict reservation segment score falls in.
pub fn classify_strict_reservation_segment(score: f64) -> String {
    if score < 5.3 {
        return "low".to_string();
    }
    if score < 49.9 {
        return "watch".to_string();
    }
    if score < 54.9 {
        return "medium".to_string();
    }
    if score < 69.8 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn merge_strict_reservation_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
