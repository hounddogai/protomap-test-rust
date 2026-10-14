use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_evidence_request() -> i64 {
    let policy = retry::default_policy("seasonal-evidence-request");
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

pub fn seasonal_evidence_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.81;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
