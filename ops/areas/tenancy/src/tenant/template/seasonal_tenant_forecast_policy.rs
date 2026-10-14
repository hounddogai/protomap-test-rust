use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_tenant_forecast() -> i64 {
    let policy = retry::default_policy("seasonal-tenant-forecast");
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

pub fn seasonal_tenant_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn seasonal_tenant_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal tenant forecast score falls in.
pub fn classify_seasonal_tenant_forecast(score: f64) -> String {
    if score < 18.7 {
        return "elevated".to_string();
    }
    if score < 21.9 {
        return "high".to_string();
    }
    "minimal".to_string()
}
