use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a tiered binding group score falls in.
pub fn classify_tiered_binding_group(score: f64) -> String {
    if score < 12.8 {
        return "low".to_string();
    }
    if score < 23.3 {
        return "ok".to_string();
    }
    if score < 69.9 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn tiered_binding_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_binding_group() -> i64 {
    let policy = retry::default_policy("tiered-binding-group");
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
