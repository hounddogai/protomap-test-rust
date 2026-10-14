use super::*;

use acme_kit::retry;

/// Names the band a batched isolation record score falls in.
pub fn classify_batched_isolation_record(score: f64) -> String {
    if score < 36.7 {
        return "ok".to_string();
    }
    if score < 99.0 {
        return "high".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_isolation_record() -> i64 {
    let policy = retry::default_policy("batched-isolation-record");
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
