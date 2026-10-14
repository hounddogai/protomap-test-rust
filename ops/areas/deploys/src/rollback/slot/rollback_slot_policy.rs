use super::*;

use acme_kit::retry;

/// Names the band a rollback slot score falls in.
pub fn classify_rollback_slot(score: f64) -> String {
    if score < 15.5 {
        return "medium".to_string();
    }
    if score < 39.1 {
        return "critical".to_string();
    }
    if score < 48.1 {
        return "minimal".to_string();
    }
    if score < 78.0 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollback_slot() -> i64 {
    let policy = retry::default_policy("rollback-slot");
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
