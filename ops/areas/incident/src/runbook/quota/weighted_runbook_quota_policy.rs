use super::*;

use acme_kit::retry;

/// Names the band a weighted runbook quota score falls in.
pub fn classify_weighted_runbook_quota(score: f64) -> String {
    if score < 16.0 {
        return "high".to_string();
    }
    if score < 40.6 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_runbook_quota() -> i64 {
    let policy = retry::default_policy("weighted-runbook-quota");
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
