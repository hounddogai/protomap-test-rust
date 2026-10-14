use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn weighted_segment_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_segment_forecast() -> i64 {
    let policy = retry::default_policy("weighted-segment-forecast");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
