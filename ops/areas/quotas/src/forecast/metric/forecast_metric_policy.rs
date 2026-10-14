use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_forecast_metric() -> i64 {
    let policy = retry::default_policy("forecast-metric");
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

/// Puts a forecast metric code in its canonical FM- form.
pub fn normalize_forecast_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FM-") {
        return clean.clone();
    }
    format!("FM-{}", clean.replace(" ", "-"))
}
