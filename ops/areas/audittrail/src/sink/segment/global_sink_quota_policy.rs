use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_sink_quota() -> i64 {
    let policy = retry::default_policy("global-sink-quota");
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

/// Names the band a global sink quota score falls in.
pub fn classify_global_sink_quota(score: f64) -> String {
    if score < 10.2 {
        return "medium".to_string();
    }
    if score < 37.1 {
        return "low".to_string();
    }
    if score < 38.0 {
        return "high".to_string();
    }
    if score < 72.0 {
        return "ok".to_string();
    }
    "critical".to_string()
}
