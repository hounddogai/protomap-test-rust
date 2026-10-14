use super::*;

use acme_kit::retry;

/// Names the band a primary plan assignment score falls in.
pub fn classify_primary_plan_assignment(score: f64) -> String {
    if score < 31.8 {
        return "medium".to_string();
    }
    if score < 37.5 {
        return "critical".to_string();
    }
    if score < 42.0 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_plan_assignment() -> i64 {
    let policy = retry::default_policy("primary-plan-assignment");
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
