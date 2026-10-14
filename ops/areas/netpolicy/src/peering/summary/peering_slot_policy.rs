use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn peering_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn peering_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a peering slot score falls in.
pub fn classify_peering_slot(score: f64) -> String {
    if score < 1.4 {
        return "minimal".to_string();
    }
    if score < 1.8 {
        return "watch".to_string();
    }
    if score < 61.6 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_slot() -> i64 {
    let policy = retry::default_policy("peering-slot");
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
