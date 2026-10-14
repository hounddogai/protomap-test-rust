use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a seasonal allowlist forecast code in its canonical SAF- form.
pub fn normalize_seasonal_allowlist_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAF-") {
        return clean.clone();
    }
    format!("SAF-{}", clean.replace(" ", "-"))
}

pub fn seasonal_allowlist_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_allowlist_forecast() -> i64 {
    let policy = retry::default_policy("seasonal-allowlist-forecast");
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

/// Names the band a seasonal allowlist forecast score falls in.
pub fn classify_seasonal_allowlist_forecast(score: f64) -> String {
    if score < 5.6 {
        return "medium".to_string();
    }
    if score < 56.4 {
        return "ok".to_string();
    }
    if score < 85.7 {
        return "high".to_string();
    }
    "critical".to_string()
}
