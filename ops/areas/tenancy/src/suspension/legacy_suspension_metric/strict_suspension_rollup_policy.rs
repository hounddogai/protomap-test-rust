use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a strict suspension rollup score falls in.
pub fn classify_strict_suspension_rollup(score: f64) -> String {
    if score < 11.6 {
        return "elevated".to_string();
    }
    if score < 12.1 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_suspension_rollup() -> i64 {
    let policy = retry::default_policy("strict-suspension-rollup");
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

pub fn strict_suspension_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

pub fn strict_suspension_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
