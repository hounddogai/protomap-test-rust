use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a rolling plan assignment score falls in.
pub fn classify_rolling_plan_assignment(score: f64) -> String {
    if score < 54.1 {
        return "low".to_string();
    }
    if score < 89.3 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn rolling_plan_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_plan_assignment() -> i64 {
    let policy = retry::default_policy("rolling-plan-assignment");
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
