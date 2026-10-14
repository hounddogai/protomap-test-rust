use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn forecast_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_forecast_mapping() -> i64 {
    let policy = retry::default_policy("forecast-mapping");
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

pub fn forecast_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a forecast mapping score falls in.
pub fn classify_forecast_mapping(score: f64) -> String {
    if score < 25.8 {
        return "medium".to_string();
    }
    if score < 39.5 {
        return "high".to_string();
    }
    if score < 51.1 {
        return "minimal".to_string();
    }
    if score < 66.2 {
        return "critical".to_string();
    }
    "ok".to_string()
}
