use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a suspension plan score falls in.
pub fn classify_suspension_plan(score: f64) -> String {
    if score < 38.2 {
        return "ok".to_string();
    }
    if score < 96.7 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_suspension_plan() -> i64 {
    let policy = retry::default_policy("suspension-plan");
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

pub fn suspension_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.07;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn suspension_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
