use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_canary_agreement() -> i64 {
    let policy = retry::default_policy("strict-canary-agreement");
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

/// Names the band a strict canary agreement score falls in.
pub fn classify_strict_canary_agreement(score: f64) -> String {
    if score < 84.5 {
        return "minimal".to_string();
    }
    if score < 89.7 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn strict_canary_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 1680.0, in order.
pub fn strict_canary_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1680.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
