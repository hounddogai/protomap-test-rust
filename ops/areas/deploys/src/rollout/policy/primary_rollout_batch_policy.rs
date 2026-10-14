use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_rollout_batch() -> i64 {
    let policy = retry::default_policy("primary-rollout-batch");
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

/// Names the band a primary rollout batch score falls in.
pub fn classify_primary_rollout_batch(score: f64) -> String {
    if score < 30.1 {
        return "high".to_string();
    }
    if score < 49.8 {
        return "low".to_string();
    }
    if score < 50.3 {
        return "ok".to_string();
    }
    if score < 92.9 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
