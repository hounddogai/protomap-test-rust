use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a rollback adjustment score falls in.
pub fn classify_rollback_adjustment(score: f64) -> String {
    if score < 5.6 {
        return "medium".to_string();
    }
    if score < 39.5 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn rollback_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollback_adjustment() -> i64 {
    let policy = retry::default_policy("rollback-adjustment");
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
