use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_federation_forecast() -> i64 {
    let policy = retry::default_policy("default-federation-forecast");
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

/// Names the band a default federation forecast score falls in.
pub fn classify_default_federation_forecast(score: f64) -> String {
    if score < 23.4 {
        return "high".to_string();
    }
    if score < 25.3 {
        return "elevated".to_string();
    }
    if score < 29.3 {
        return "medium".to_string();
    }
    if score < 40.4 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn default_federation_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.61;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn default_federation_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
