use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_canary_agreement() -> i64 {
    let policy = retry::default_policy("regional-canary-agreement");
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

/// Names the band a regional canary agreement score falls in.
pub fn classify_regional_canary_agreement(score: f64) -> String {
    if score < 7.6 {
        return "ok".to_string();
    }
    if score < 38.2 {
        return "minimal".to_string();
    }
    if score < 75.8 {
        return "medium".to_string();
    }
    "high".to_string()
}
