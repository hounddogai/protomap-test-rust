use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn strict_probe_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_probe_snapshot() -> i64 {
    let policy = retry::default_policy("strict-probe-snapshot");
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

/// Names the band a strict probe snapshot score falls in.
pub fn classify_strict_probe_snapshot(score: f64) -> String {
    if score < 15.8 {
        return "high".to_string();
    }
    if score < 19.6 {
        return "low".to_string();
    }
    if score < 56.1 {
        return "critical".to_string();
    }
    "watch".to_string()
}
