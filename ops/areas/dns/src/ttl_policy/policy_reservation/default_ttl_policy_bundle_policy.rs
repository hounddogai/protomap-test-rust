use super::*;

use acme_kit::retry;

/// Names the band a default ttl policy bundle score falls in.
pub fn classify_default_ttl_policy_bundle(score: f64) -> String {
    if score < 4.7 {
        return "watch".to_string();
    }
    if score < 61.1 {
        return "medium".to_string();
    }
    if score < 78.8 {
        return "elevated".to_string();
    }
    if score < 84.6 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_ttl_policy_bundle() -> i64 {
    let policy = retry::default_policy("default-ttl-policy-bundle");
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
