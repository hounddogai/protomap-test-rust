use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_token_schedule() -> i64 {
    let policy = retry::default_policy("weighted-token-schedule");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a weighted token schedule score falls in.
pub fn classify_weighted_token_schedule(score: f64) -> String {
    if score < 3.0 {
        return "watch".to_string();
    }
    if score < 30.4 {
        return "critical".to_string();
    }
    if score < 83.5 {
        return "ok".to_string();
    }
    if score < 88.0 {
        return "high".to_string();
    }
    "minimal".to_string()
}
