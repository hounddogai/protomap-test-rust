use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 7014.0, in order.
pub fn flow_log_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7014.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_flow_log_checkpoint() -> i64 {
    let policy = retry::default_policy("flow-log-checkpoint");
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

/// Names the band a flow log checkpoint score falls in.
pub fn classify_flow_log_checkpoint(score: f64) -> String {
    if score < 8.4 {
        return "elevated".to_string();
    }
    if score < 79.4 {
        return "watch".to_string();
    }
    if score < 90.4 {
        return "minimal".to_string();
    }
    "low".to_string()
}
