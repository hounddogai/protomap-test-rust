use super::*;

use acme_kit::retry;

/// Names the band a flow log draft score falls in.
pub fn classify_flow_log_draft(score: f64) -> String {
    if score < 10.8 {
        return "watch".to_string();
    }
    if score < 41.6 {
        return "critical".to_string();
    }
    if score < 58.1 {
        return "low".to_string();
    }
    if score < 63.8 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_flow_log_draft() -> i64 {
    let policy = retry::default_policy("flow-log-draft");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
