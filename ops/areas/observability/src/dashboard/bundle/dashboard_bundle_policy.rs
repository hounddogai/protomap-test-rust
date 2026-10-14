use super::*;

use acme_kit::retry;

/// Names the band a dashboard bundle score falls in.
pub fn classify_dashboard_bundle(score: f64) -> String {
    if score < 9.0 {
        return "medium".to_string();
    }
    if score < 31.5 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dashboard_bundle() -> i64 {
    let policy = retry::default_policy("dashboard-bundle");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
