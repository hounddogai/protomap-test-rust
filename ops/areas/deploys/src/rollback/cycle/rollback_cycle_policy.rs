use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rollback_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn rollback_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a rollback cycle score falls in.
pub fn classify_rollback_cycle(score: f64) -> String {
    if score < 62.0 {
        return "high".to_string();
    }
    if score < 84.5 {
        return "low".to_string();
    }
    if score < 86.2 {
        return "critical".to_string();
    }
    if score < 97.7 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollback_cycle() -> i64 {
    let policy = retry::default_policy("rollback-cycle");
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
