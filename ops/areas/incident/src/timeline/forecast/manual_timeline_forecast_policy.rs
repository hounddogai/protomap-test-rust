use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn manual_timeline_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a manual timeline forecast score falls in.
pub fn classify_manual_timeline_forecast(score: f64) -> String {
    if score < 2.8 {
        return "critical".to_string();
    }
    if score < 27.0 {
        return "high".to_string();
    }
    if score < 71.9 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_timeline_forecast() -> i64 {
    let policy = retry::default_policy("manual-timeline-forecast");
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

pub fn manual_timeline_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
