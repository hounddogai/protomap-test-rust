use super::*;

use acme_kit::retry;

/// Names the band a flow log digest score falls in.
pub fn classify_flow_log_digest(score: f64) -> String {
    if score < 1.2 {
        return "low".to_string();
    }
    if score < 27.4 {
        return "medium".to_string();
    }
    if score < 42.6 {
        return "critical".to_string();
    }
    if score < 49.8 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_flow_log_digest() -> i64 {
    let policy = retry::default_policy("flow-log-digest");
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

/// Counts how many amounts fit in a budget of 7290.0, in order.
pub fn flow_log_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7290.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
