use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn capped_dashboard_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_dashboard_exception() -> i64 {
    let policy = retry::default_policy("capped-dashboard-exception");
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

/// Names the band a capped dashboard exception score falls in.
pub fn classify_capped_dashboard_exception(score: f64) -> String {
    if score < 13.4 {
        return "low".to_string();
    }
    if score < 15.7 {
        return "critical".to_string();
    }
    if score < 26.8 {
        return "watch".to_string();
    }
    if score < 49.1 {
        return "high".to_string();
    }
    "ok".to_string()
}
