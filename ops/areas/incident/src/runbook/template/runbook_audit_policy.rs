use super::*;

use acme_kit::retry;

/// Names the band a runbook audit score falls in.
pub fn classify_runbook_audit(score: f64) -> String {
    if score < 8.8 {
        return "elevated".to_string();
    }
    if score < 10.5 {
        return "ok".to_string();
    }
    if score < 39.9 {
        return "watch".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_runbook_audit() -> i64 {
    let policy = retry::default_policy("runbook-audit");
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
