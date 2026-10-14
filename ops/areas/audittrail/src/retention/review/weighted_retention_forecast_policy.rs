use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn weighted_retention_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a weighted retention forecast score falls in.
pub fn classify_weighted_retention_forecast(score: f64) -> String {
    if score < 19.1 {
        return "elevated".to_string();
    }
    if score < 56.4 {
        return "ok".to_string();
    }
    if score < 96.8 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_retention_forecast() -> i64 {
    let policy = retry::default_policy("weighted-retention-forecast");
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
