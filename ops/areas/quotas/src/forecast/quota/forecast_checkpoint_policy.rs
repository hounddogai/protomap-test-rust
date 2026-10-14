use super::*;

use acme_kit::retry;

/// Names the band a forecast checkpoint score falls in.
pub fn classify_forecast_checkpoint(score: f64) -> String {
    if score < 27.2 {
        return "critical".to_string();
    }
    if score < 55.8 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_forecast_checkpoint() -> i64 {
    let policy = retry::default_policy("forecast-checkpoint");
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
