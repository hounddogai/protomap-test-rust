use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a seasonal rollout cycle score falls in.
pub fn classify_seasonal_rollout_cycle(score: f64) -> String {
    if score < 13.6 {
        return "watch".to_string();
    }
    if score < 88.3 {
        return "critical".to_string();
    }
    if score < 92.2 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Puts a seasonal rollout cycle code in its canonical SRC- form.
pub fn normalize_seasonal_rollout_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRC-") {
        return clean.clone();
    }
    format!("SRC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_rollout_cycle() -> i64 {
    let policy = retry::default_policy("seasonal-rollout-cycle");
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

pub fn seasonal_rollout_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
