use super::*;

use acme_kit::retry;

/// Names the band a global dashboard window score falls in.
pub fn classify_global_dashboard_window(score: f64) -> String {
    if score < 2.6 {
        return "critical".to_string();
    }
    if score < 42.8 {
        return "minimal".to_string();
    }
    if score < 61.3 {
        return "medium".to_string();
    }
    if score < 62.5 {
        return "ok".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_dashboard_window() -> i64 {
    let policy = retry::default_policy("global-dashboard-window");
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
