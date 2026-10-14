use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_cycle() -> i64 {
    let policy = retry::default_policy("escalation-cycle");
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

pub fn escalation_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a escalation cycle score falls in.
pub fn classify_escalation_cycle(score: f64) -> String {
    if score < 17.5 {
        return "medium".to_string();
    }
    if score < 39.3 {
        return "ok".to_string();
    }
    if score < 97.4 {
        return "watch".to_string();
    }
    "critical".to_string()
}
