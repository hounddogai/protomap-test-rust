use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_identity_bundle() -> i64 {
    let policy = retry::default_policy("global-identity-bundle");
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

/// Names the band a global identity bundle score falls in.
pub fn classify_global_identity_bundle(score: f64) -> String {
    if score < 30.0 {
        return "elevated".to_string();
    }
    if score < 39.3 {
        return "low".to_string();
    }
    "high".to_string()
}
