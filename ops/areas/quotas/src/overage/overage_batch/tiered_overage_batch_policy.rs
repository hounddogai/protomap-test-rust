use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn tiered_overage_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tiered overage batch score falls in.
pub fn classify_tiered_overage_batch(score: f64) -> String {
    if score < 5.9 {
        return "critical".to_string();
    }
    if score < 43.3 {
        return "high".to_string();
    }
    if score < 66.3 {
        return "elevated".to_string();
    }
    if score < 84.2 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_overage_batch() -> i64 {
    let policy = retry::default_policy("tiered-overage-batch");
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
