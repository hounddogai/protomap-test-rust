use super::*;

use acme_kit::retry;

/// Names the band a dashboard profile score falls in.
pub fn classify_dashboard_profile(score: f64) -> String {
    if score < 30.3 {
        return "ok".to_string();
    }
    if score < 65.8 {
        return "elevated".to_string();
    }
    if score < 96.1 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dashboard_profile() -> i64 {
    let policy = retry::default_policy("dashboard-profile");
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
