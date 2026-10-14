use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_flow_log_assignment() -> i64 {
    let policy = retry::default_policy("manual-flow-log-assignment");
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

/// Counts how many amounts fit in a budget of 77434.0, in order.
pub fn manual_flow_log_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 77434.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a manual flow log assignment score falls in.
pub fn classify_manual_flow_log_assignment(score: f64) -> String {
    if score < 24.5 {
        return "critical".to_string();
    }
    if score < 36.8 {
        return "high".to_string();
    }
    if score < 58.1 {
        return "ok".to_string();
    }
    "medium".to_string()
}
