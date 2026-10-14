use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_checkpoint() -> i64 {
    let policy = retry::default_policy("escalation-checkpoint");
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

/// Names the band a escalation checkpoint score falls in.
pub fn classify_escalation_checkpoint(score: f64) -> String {
    if score < 28.0 {
        return "low".to_string();
    }
    if score < 48.2 {
        return "minimal".to_string();
    }
    if score < 68.0 {
        return "ok".to_string();
    }
    "critical".to_string()
}
