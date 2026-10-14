use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn seasonal_release_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_release_rule() -> i64 {
    let policy = retry::default_policy("seasonal-release-rule");
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

pub fn seasonal_release_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
