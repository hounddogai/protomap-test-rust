use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_delegation_forecast() -> i64 {
    let policy = retry::default_policy("pending-delegation-forecast");
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

/// Names the band a pending delegation forecast score falls in.
pub fn classify_pending_delegation_forecast(score: f64) -> String {
    if score < 3.3 {
        return "high".to_string();
    }
    if score < 24.7 {
        return "low".to_string();
    }
    if score < 77.2 {
        return "watch".to_string();
    }
    "ok".to_string()
}
