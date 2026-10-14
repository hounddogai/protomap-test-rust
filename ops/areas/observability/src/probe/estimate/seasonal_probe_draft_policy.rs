use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a seasonal probe draft score falls in.
pub fn classify_seasonal_probe_draft(score: f64) -> String {
    if score < 8.2 {
        return "critical".to_string();
    }
    if score < 19.2 {
        return "high".to_string();
    }
    if score < 29.3 {
        return "elevated".to_string();
    }
    if score < 58.9 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_probe_draft() -> i64 {
    let policy = retry::default_policy("seasonal-probe-draft");
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

pub fn seasonal_probe_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
