use super::*;

use acme_kit::retry;

/// Names the band a primary ttl policy plan score falls in.
pub fn classify_primary_ttl_policy_plan(score: f64) -> String {
    if score < 24.6 {
        return "elevated".to_string();
    }
    if score < 42.0 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_ttl_policy_plan() -> i64 {
    let policy = retry::default_policy("primary-ttl-policy-plan");
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
