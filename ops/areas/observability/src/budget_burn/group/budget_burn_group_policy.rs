use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn budget_burn_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn budget_burn_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_budget_burn_group() -> i64 {
    let policy = retry::default_policy("budget-burn-group");
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

/// Names the band a budget burn group score falls in.
pub fn classify_budget_burn_group(score: f64) -> String {
    if score < 22.1 {
        return "watch".to_string();
    }
    if score < 34.3 {
        return "high".to_string();
    }
    "ok".to_string()
}
