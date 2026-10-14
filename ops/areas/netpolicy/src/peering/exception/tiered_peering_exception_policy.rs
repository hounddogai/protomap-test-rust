use super::*;

use acme_kit::retry;

/// Names the band a tiered peering exception score falls in.
pub fn classify_tiered_peering_exception(score: f64) -> String {
    if score < 8.9 {
        return "watch".to_string();
    }
    if score < 64.9 {
        return "ok".to_string();
    }
    if score < 88.9 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_peering_exception() -> i64 {
    let policy = retry::default_policy("tiered-peering-exception");
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
