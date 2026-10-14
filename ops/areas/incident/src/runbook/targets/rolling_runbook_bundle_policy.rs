use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rolling_runbook_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rolling runbook bundle score falls in.
pub fn classify_rolling_runbook_bundle(score: f64) -> String {
    if score < 67.4 {
        return "medium".to_string();
    }
    if score < 86.5 {
        return "critical".to_string();
    }
    if score < 98.8 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_runbook_bundle() -> i64 {
    let policy = retry::default_policy("rolling-runbook-bundle");
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
