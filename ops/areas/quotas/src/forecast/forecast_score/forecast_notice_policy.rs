use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn forecast_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a forecast notice score falls in.
pub fn classify_forecast_notice(score: f64) -> String {
    if score < 56.2 {
        return "critical".to_string();
    }
    if score < 57.2 {
        return "low".to_string();
    }
    if score < 77.7 {
        return "ok".to_string();
    }
    if score < 81.4 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

pub fn forecast_notice_span(start: i64, end: i64) -> i64 {
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
pub fn probe_forecast_notice() -> i64 {
    let policy = retry::default_policy("forecast-notice");
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
