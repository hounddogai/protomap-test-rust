use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rolling_budget_burn_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_budget_burn_template() -> i64 {
    let policy = retry::default_policy("rolling-budget-burn-template");
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
