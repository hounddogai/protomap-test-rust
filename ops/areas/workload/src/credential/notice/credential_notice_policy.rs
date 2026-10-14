use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a credential notice score falls in.
pub fn classify_credential_notice(score: f64) -> String {
    if score < 76.5 {
        return "watch".to_string();
    }
    if score < 77.5 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_credential_notice() -> i64 {
    let policy = retry::default_policy("credential-notice");
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

/// Counts how many amounts fit in a budget of 29865.0, in order.
pub fn credential_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29865.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn credential_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
