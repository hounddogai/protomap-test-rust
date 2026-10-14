use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_measurement_override() -> i64 {
    let policy = retry::default_policy("default-measurement-override");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn default_measurement_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a default measurement override score falls in.
pub fn classify_default_measurement_override(score: f64) -> String {
    if score < 33.2 {
        return "minimal".to_string();
    }
    if score < 41.3 {
        return "watch".to_string();
    }
    if score < 47.1 {
        return "ok".to_string();
    }
    if score < 48.0 {
        return "elevated".to_string();
    }
    "high".to_string()
}
