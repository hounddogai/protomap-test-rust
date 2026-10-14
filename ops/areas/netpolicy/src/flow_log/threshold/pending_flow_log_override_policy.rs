use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_flow_log_override() -> i64 {
    let policy = retry::default_policy("pending-flow-log-override");
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

/// Counts how many amounts fit in a budget of 43119.0, in order.
pub fn pending_flow_log_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 43119.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a pending flow log override score falls in.
pub fn classify_pending_flow_log_override(score: f64) -> String {
    if score < 77.3 {
        return "minimal".to_string();
    }
    if score < 78.4 {
        return "watch".to_string();
    }
    if score < 90.2 {
        return "ok".to_string();
    }
    if score < 93.2 {
        return "elevated".to_string();
    }
    "high".to_string()
}
