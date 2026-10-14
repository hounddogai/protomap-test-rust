use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_grant_baseline() -> i64 {
    let policy = retry::default_policy("seasonal-grant-baseline");
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

/// Names the band a seasonal grant baseline score falls in.
pub fn classify_seasonal_grant_baseline(score: f64) -> String {
    if score < 59.0 {
        return "elevated".to_string();
    }
    if score < 88.4 {
        return "high".to_string();
    }
    if score < 95.8 {
        return "low".to_string();
    }
    "ok".to_string()
}
