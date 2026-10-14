use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_forecast_request() -> i64 {
    let policy = retry::default_policy("regional-forecast-request");
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

/// Names the band a regional forecast request score falls in.
pub fn classify_regional_forecast_request(score: f64) -> String {
    if score < 11.4 {
        return "minimal".to_string();
    }
    if score < 39.7 {
        return "ok".to_string();
    }
    if score < 79.3 {
        return "critical".to_string();
    }
    if score < 84.9 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn regional_forecast_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
