use super::*;

use acme_kit::retry;

pub fn freeze_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Puts a freeze forecast code in its canonical FF- form.
pub fn normalize_freeze_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FF-") {
        return clean.clone();
    }
    format!("FF-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_forecast() -> i64 {
    let policy = retry::default_policy("freeze-forecast");
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

/// Names the band a freeze forecast score falls in.
pub fn classify_freeze_forecast(score: f64) -> String {
    if score < 42.6 {
        return "low".to_string();
    }
    if score < 45.7 {
        return "critical".to_string();
    }
    "high".to_string()
}
