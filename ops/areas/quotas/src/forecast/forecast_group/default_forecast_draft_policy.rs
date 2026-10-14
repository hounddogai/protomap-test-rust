use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn default_forecast_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_forecast_draft() -> i64 {
    let policy = retry::default_policy("default-forecast-draft");
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

/// Names the band a default forecast draft score falls in.
pub fn classify_default_forecast_draft(score: f64) -> String {
    if score < 20.6 {
        return "elevated".to_string();
    }
    if score < 61.3 {
        return "ok".to_string();
    }
    if score < 96.6 {
        return "watch".to_string();
    }
    "medium".to_string()
}
