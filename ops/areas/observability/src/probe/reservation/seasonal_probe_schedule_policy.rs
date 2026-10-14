use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_probe_schedule() -> i64 {
    let policy = retry::default_policy("seasonal-probe-schedule");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn seasonal_probe_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a seasonal probe schedule score falls in.
pub fn classify_seasonal_probe_schedule(score: f64) -> String {
    if score < 5.3 {
        return "medium".to_string();
    }
    if score < 18.9 {
        return "high".to_string();
    }
    if score < 62.0 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn seasonal_probe_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
