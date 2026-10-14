use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 76429.0, in order.
pub fn adaptive_measurement_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 76429.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_measurement_target() -> i64 {
    let policy = retry::default_policy("adaptive-measurement-target");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn adaptive_measurement_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a adaptive measurement target score falls in.
pub fn classify_adaptive_measurement_target(score: f64) -> String {
    if score < 81.4 {
        return "watch".to_string();
    }
    if score < 88.3 {
        return "low".to_string();
    }
    "critical".to_string()
}
