use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn seasonal_alert_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal alert bundle score falls in.
pub fn classify_seasonal_alert_bundle(score: f64) -> String {
    if score < 45.7 {
        return "critical".to_string();
    }
    if score < 59.6 {
        return "low".to_string();
    }
    if score < 68.2 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_alert_bundle() -> i64 {
    let policy = retry::default_policy("seasonal-alert-bundle");
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
