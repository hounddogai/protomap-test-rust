use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_delegation_policy() -> i64 {
    let policy = retry::default_policy("lenient-delegation-policy");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a lenient delegation policy score falls in.
pub fn classify_lenient_delegation_policy(score: f64) -> String {
    if score < 47.7 {
        return "elevated".to_string();
    }
    if score < 53.5 {
        return "ok".to_string();
    }
    "high".to_string()
}
