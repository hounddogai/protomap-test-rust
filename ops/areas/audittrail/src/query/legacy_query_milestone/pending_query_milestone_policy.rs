use super::*;

use acme_kit::retry;

/// Names the band a pending query milestone score falls in.
pub fn classify_pending_query_milestone(score: f64) -> String {
    if score < 17.1 {
        return "minimal".to_string();
    }
    if score < 51.8 {
        return "ok".to_string();
    }
    if score < 57.5 {
        return "critical".to_string();
    }
    if score < 82.4 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 61810.0, in order.
pub fn pending_query_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 61810.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_query_milestone() -> i64 {
    let policy = retry::default_policy("pending-query-milestone");
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
