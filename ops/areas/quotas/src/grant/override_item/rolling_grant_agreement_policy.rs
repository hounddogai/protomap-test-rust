use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a rolling grant agreement score falls in.
pub fn classify_rolling_grant_agreement(score: f64) -> String {
    if score < 24.2 {
        return "elevated".to_string();
    }
    if score < 37.0 {
        return "critical".to_string();
    }
    if score < 64.3 {
        return "high".to_string();
    }
    if score < 66.2 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 69874.0, in order.
pub fn rolling_grant_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69874.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn rolling_grant_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.19;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_grant_agreement() -> i64 {
    let policy = retry::default_policy("rolling-grant-agreement");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
