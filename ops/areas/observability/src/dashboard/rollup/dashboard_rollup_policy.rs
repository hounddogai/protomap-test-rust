use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a dashboard rollup score falls in.
pub fn classify_dashboard_rollup(score: f64) -> String {
    if score < 3.7 {
        return "minimal".to_string();
    }
    if score < 52.3 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dashboard_rollup() -> i64 {
    let policy = retry::default_policy("dashboard-rollup");
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

pub fn dashboard_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
