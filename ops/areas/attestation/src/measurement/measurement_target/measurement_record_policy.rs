use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn measurement_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_measurement_record() -> i64 {
    let policy = retry::default_policy("measurement-record");
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

/// Names the band a measurement record score falls in.
pub fn classify_measurement_record(score: f64) -> String {
    if score < 5.5 {
        return "critical".to_string();
    }
    if score < 14.1 {
        return "elevated".to_string();
    }
    if score < 38.9 {
        return "ok".to_string();
    }
    if score < 75.6 {
        return "watch".to_string();
    }
    "medium".to_string()
}
