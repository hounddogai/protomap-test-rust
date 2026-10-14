use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_measurement_cycle() -> i64 {
    let policy = retry::default_policy("measurement-cycle");
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

pub fn measurement_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a measurement cycle score falls in.
pub fn classify_measurement_cycle(score: f64) -> String {
    if score < 60.8 {
        return "low".to_string();
    }
    if score < 63.1 {
        return "ok".to_string();
    }
    if score < 74.6 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn measurement_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
