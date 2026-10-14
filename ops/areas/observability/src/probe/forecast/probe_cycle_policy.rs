use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn probe_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_probe_cycle() -> i64 {
    let policy = retry::default_policy("probe-cycle");
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

pub fn probe_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
