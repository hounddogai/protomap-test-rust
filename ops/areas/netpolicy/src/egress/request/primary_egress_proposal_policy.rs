use super::*;

use acme_kit::retry;

/// Names the band a primary egress proposal score falls in.
pub fn classify_primary_egress_proposal(score: f64) -> String {
    if score < 21.1 {
        return "elevated".to_string();
    }
    if score < 63.5 {
        return "ok".to_string();
    }
    if score < 69.0 {
        return "medium".to_string();
    }
    if score < 84.0 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_egress_proposal() -> i64 {
    let policy = retry::default_policy("primary-egress-proposal");
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
