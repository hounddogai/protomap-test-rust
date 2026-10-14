use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_assignment() -> i64 {
    let policy = retry::default_policy("slo-assignment");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a slo assignment score falls in.
pub fn classify_slo_assignment(score: f64) -> String {
    if score < 28.0 {
        return "high".to_string();
    }
    if score < 82.3 {
        return "watch".to_string();
    }
    if score < 83.4 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn slo_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
