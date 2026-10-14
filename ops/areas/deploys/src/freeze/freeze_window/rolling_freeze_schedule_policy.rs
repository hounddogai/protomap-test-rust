use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rolling_freeze_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_freeze_schedule() -> i64 {
    let policy = retry::default_policy("rolling-freeze-schedule");
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

/// Names the band a rolling freeze schedule score falls in.
pub fn classify_rolling_freeze_schedule(score: f64) -> String {
    if score < 22.1 {
        return "low".to_string();
    }
    if score < 45.8 {
        return "elevated".to_string();
    }
    if score < 63.8 {
        return "watch".to_string();
    }
    "critical".to_string()
}
