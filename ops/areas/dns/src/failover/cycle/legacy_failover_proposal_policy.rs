use super::*;

use acme_kit::retry;

/// Names the band a legacy failover proposal score falls in.
pub fn classify_legacy_failover_proposal(score: f64) -> String {
    if score < 21.7 {
        return "critical".to_string();
    }
    if score < 42.4 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_failover_proposal() -> i64 {
    let policy = retry::default_policy("legacy-failover-proposal");
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
