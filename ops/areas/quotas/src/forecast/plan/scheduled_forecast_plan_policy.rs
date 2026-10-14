use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a scheduled forecast plan code in its canonical SFP- form.
pub fn normalize_scheduled_forecast_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFP-") {
        return clean.clone();
    }
    format!("SFP-{}", clean.replace(" ", "-"))
}

pub fn scheduled_forecast_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.01;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_forecast_plan() -> i64 {
    let policy = retry::default_policy("scheduled-forecast-plan");
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
