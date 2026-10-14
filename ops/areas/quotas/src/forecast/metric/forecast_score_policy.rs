use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn forecast_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a forecast score code in its canonical FS- form.
pub fn normalize_forecast_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FS-") {
        return clean.clone();
    }
    format!("FS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_forecast_score() -> i64 {
    let policy = retry::default_policy("forecast-score");
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
