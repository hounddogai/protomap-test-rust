use super::*;

use acme_kit::retry;

/// Names the band a ttl policy score score falls in.
pub fn classify_ttl_policy_score(score: f64) -> String {
    if score < 34.4 {
        return "elevated".to_string();
    }
    if score < 86.1 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_score() -> i64 {
    let policy = retry::default_policy("ttl-policy-score");
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
