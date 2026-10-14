use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_zone_window() -> i64 {
    let policy = retry::default_policy("zone-window");
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

/// Names the band a zone window score falls in.
pub fn classify_zone_window(score: f64) -> String {
    if score < 35.0 {
        return "elevated".to_string();
    }
    if score < 82.8 {
        return "low".to_string();
    }
    "critical".to_string()
}
