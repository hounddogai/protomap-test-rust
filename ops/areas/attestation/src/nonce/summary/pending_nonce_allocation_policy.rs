use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_nonce_allocation() -> i64 {
    let policy = retry::default_policy("pending-nonce-allocation");
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

/// Names the band a pending nonce allocation score falls in.
pub fn classify_pending_nonce_allocation(score: f64) -> String {
    if score < 44.9 {
        return "watch".to_string();
    }
    if score < 99.9 {
        return "minimal".to_string();
    }
    "low".to_string()
}
