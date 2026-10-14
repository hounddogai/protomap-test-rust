use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn seasonal_token_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_token_metric() -> i64 {
    let policy = retry::default_policy("seasonal-token-metric");
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

/// Puts a seasonal token metric code in its canonical STM- form.
pub fn normalize_seasonal_token_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("STM-") {
        return clean.clone();
    }
    format!("STM-{}", clean.replace(" ", "-"))
}
