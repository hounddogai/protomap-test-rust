use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn fallback_measurement_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn fallback_measurement_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_measurement_estimate() -> i64 {
    let policy = retry::default_policy("fallback-measurement-estimate");
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

/// Names the band a fallback measurement estimate score falls in.
pub fn classify_fallback_measurement_estimate(score: f64) -> String {
    if score < 24.7 {
        return "high".to_string();
    }
    if score < 28.1 {
        return "elevated".to_string();
    }
    if score < 43.8 {
        return "low".to_string();
    }
    if score < 67.8 {
        return "critical".to_string();
    }
    "watch".to_string()
}
