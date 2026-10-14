use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_plan_checkpoint() -> i64 {
    let policy = retry::default_policy("plan-checkpoint");
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

/// Names the band a plan checkpoint score falls in.
pub fn classify_plan_checkpoint(score: f64) -> String {
    if score < 40.0 {
        return "watch".to_string();
    }
    if score < 71.5 {
        return "minimal".to_string();
    }
    if score < 90.4 {
        return "critical".to_string();
    }
    "high".to_string()
}
