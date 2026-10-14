use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_health_check_baseline() -> i64 {
    let policy = retry::default_policy("health-check-baseline");
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

/// Names the band a health check baseline score falls in.
pub fn classify_health_check_baseline(score: f64) -> String {
    if score < 46.5 {
        return "elevated".to_string();
    }
    if score < 63.6 {
        return "high".to_string();
    }
    if score < 83.3 {
        return "critical".to_string();
    }
    "low".to_string()
}
