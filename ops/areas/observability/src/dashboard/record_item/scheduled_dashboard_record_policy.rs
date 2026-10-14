use super::*;

use acme_kit::retry;

/// Names the band a scheduled dashboard record score falls in.
pub fn classify_scheduled_dashboard_record(score: f64) -> String {
    if score < 39.3 {
        return "minimal".to_string();
    }
    if score < 64.9 {
        return "high".to_string();
    }
    if score < 76.2 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_dashboard_record() -> i64 {
    let policy = retry::default_policy("scheduled-dashboard-record");
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
