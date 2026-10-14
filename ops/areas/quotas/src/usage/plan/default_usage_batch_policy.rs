use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn default_usage_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.51;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_usage_batch() -> i64 {
    let policy = retry::default_policy("default-usage-batch");
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

/// Names the band a default usage batch score falls in.
pub fn classify_default_usage_batch(score: f64) -> String {
    if score < 19.3 {
        return "minimal".to_string();
    }
    if score < 71.4 {
        return "elevated".to_string();
    }
    if score < 91.4 {
        return "medium".to_string();
    }
    if score < 98.4 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn default_usage_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
