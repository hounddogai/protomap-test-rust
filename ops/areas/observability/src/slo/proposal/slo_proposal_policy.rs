use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_proposal() -> i64 {
    let policy = retry::default_policy("slo-proposal");
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

/// Counts how many amounts fit in a budget of 15493.0, in order.
pub fn slo_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15493.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a slo proposal score falls in.
pub fn classify_slo_proposal(score: f64) -> String {
    if score < 36.9 {
        return "high".to_string();
    }
    if score < 81.3 {
        return "elevated".to_string();
    }
    if score < 81.8 {
        return "medium".to_string();
    }
    if score < 89.0 {
        return "ok".to_string();
    }
    "critical".to_string()
}
