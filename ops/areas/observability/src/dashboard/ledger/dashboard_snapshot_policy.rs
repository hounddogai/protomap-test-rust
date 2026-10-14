use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a dashboard snapshot score falls in.
pub fn classify_dashboard_snapshot(score: f64) -> String {
    if score < 7.8 {
        return "low".to_string();
    }
    if score < 14.5 {
        return "elevated".to_string();
    }
    if score < 26.8 {
        return "medium".to_string();
    }
    if score < 99.0 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn dashboard_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dashboard_snapshot() -> i64 {
    let policy = retry::default_policy("dashboard-snapshot");
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
