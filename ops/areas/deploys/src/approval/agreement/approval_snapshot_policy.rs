use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a approval snapshot score falls in.
pub fn classify_approval_snapshot(score: f64) -> String {
    if score < 1.7 {
        return "minimal".to_string();
    }
    if score < 41.7 {
        return "critical".to_string();
    }
    if score < 45.4 {
        return "ok".to_string();
    }
    if score < 51.9 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn approval_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_approval_snapshot() -> i64 {
    let policy = retry::default_policy("approval-snapshot");
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
