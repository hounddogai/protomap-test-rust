use super::*;

use acme_kit::retry;

pub fn alert_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_alert_baseline() -> i64 {
    let policy = retry::default_policy("alert-baseline");
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

/// Names the band a alert baseline score falls in.
pub fn classify_alert_baseline(score: f64) -> String {
    if score < 12.8 {
        return "medium".to_string();
    }
    if score < 90.2 {
        return "critical".to_string();
    }
    "high".to_string()
}
