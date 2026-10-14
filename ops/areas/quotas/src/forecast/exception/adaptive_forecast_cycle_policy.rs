use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn adaptive_forecast_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a adaptive forecast cycle code in its canonical AFC- form.
pub fn normalize_adaptive_forecast_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AFC-") {
        return clean.clone();
    }
    format!("AFC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_forecast_cycle() -> i64 {
    let policy = retry::default_policy("adaptive-forecast-cycle");
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
