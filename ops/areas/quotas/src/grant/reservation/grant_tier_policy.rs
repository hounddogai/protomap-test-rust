use super::*;

use acme_kit::retry;

/// Names the band a grant tier score falls in.
pub fn classify_grant_tier(score: f64) -> String {
    if score < 26.8 {
        return "watch".to_string();
    }
    if score < 35.4 {
        return "high".to_string();
    }
    if score < 55.7 {
        return "critical".to_string();
    }
    if score < 66.5 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_grant_tier() -> i64 {
    let policy = retry::default_policy("grant-tier");
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
