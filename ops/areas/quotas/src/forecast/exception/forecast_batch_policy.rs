use super::*;

use acme_kit::retry;

/// Puts a forecast batch code in its canonical FB- form.
pub fn normalize_forecast_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FB-") {
        return clean.clone();
    }
    format!("FB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_forecast_batch() -> i64 {
    let policy = retry::default_policy("forecast-batch");
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

/// Names the band a forecast batch score falls in.
pub fn classify_forecast_batch(score: f64) -> String {
    if score < 19.8 {
        return "low".to_string();
    }
    if score < 61.1 {
        return "ok".to_string();
    }
    if score < 67.3 {
        return "critical".to_string();
    }
    "watch".to_string()
}
