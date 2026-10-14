use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a rolling canary rollup code in its canonical RCR- form.
pub fn normalize_rolling_canary_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RCR-") {
        return clean.clone();
    }
    format!("RCR-{}", clean.replace(" ", "-"))
}

pub fn rolling_canary_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a rolling canary rollup score falls in.
pub fn classify_rolling_canary_rollup(score: f64) -> String {
    if score < 12.0 {
        return "high".to_string();
    }
    if score < 47.5 {
        return "low".to_string();
    }
    if score < 76.5 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_canary_rollup() -> i64 {
    let policy = retry::default_policy("rolling-canary-rollup");
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
