use super::*;

use acme_kit::retry;

/// Names the band a egress proposal score falls in.
pub fn classify_egress_proposal(score: f64) -> String {
    if score < 47.8 {
        return "ok".to_string();
    }
    if score < 68.0 {
        return "medium".to_string();
    }
    if score < 71.0 {
        return "low".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 90016.0, in order.
pub fn egress_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90016.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_egress_proposal() -> i64 {
    let policy = retry::default_policy("egress-proposal");
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
