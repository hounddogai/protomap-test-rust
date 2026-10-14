use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_escalation_audit() -> i64 {
    let policy = retry::default_policy("rolling-escalation-audit");
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

pub fn rolling_escalation_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rolling escalation audit score falls in.
pub fn classify_rolling_escalation_audit(score: f64) -> String {
    if score < 28.0 {
        return "critical".to_string();
    }
    if score < 96.0 {
        return "medium".to_string();
    }
    "watch".to_string()
}
