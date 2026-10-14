use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_flow_log_score() -> i64 {
    let policy = retry::default_policy("global-flow-log-score");
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

/// Names the band a global flow log score score falls in.
pub fn classify_global_flow_log_score(score: f64) -> String {
    if score < 61.1 {
        return "critical".to_string();
    }
    if score < 67.9 {
        return "medium".to_string();
    }
    if score < 71.6 {
        return "watch".to_string();
    }
    if score < 98.5 {
        return "elevated".to_string();
    }
    "low".to_string()
}
