use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn regional_delegation_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_delegation_audit() -> i64 {
    let policy = retry::default_policy("regional-delegation-audit");
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

/// Names the band a regional delegation audit score falls in.
pub fn classify_regional_delegation_audit(score: f64) -> String {
    if score < 8.6 {
        return "minimal".to_string();
    }
    if score < 49.2 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn regional_delegation_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.15;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
