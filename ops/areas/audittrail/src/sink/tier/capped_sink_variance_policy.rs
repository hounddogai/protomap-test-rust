use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_sink_variance() -> i64 {
    let policy = retry::default_policy("capped-sink-variance");
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

pub fn capped_sink_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.23;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a capped sink variance score falls in.
pub fn classify_capped_sink_variance(score: f64) -> String {
    if score < 59.0 {
        return "low".to_string();
    }
    if score < 70.8 {
        return "watch".to_string();
    }
    if score < 90.7 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 21804.0, in order.
pub fn capped_sink_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 21804.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
