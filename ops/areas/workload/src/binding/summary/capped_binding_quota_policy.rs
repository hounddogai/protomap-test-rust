use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_binding_quota() -> i64 {
    let policy = retry::default_policy("capped-binding-quota");
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

/// Names the band a capped binding quota score falls in.
pub fn classify_capped_binding_quota(score: f64) -> String {
    if score < 31.3 {
        return "minimal".to_string();
    }
    if score < 54.0 {
        return "low".to_string();
    }
    if score < 62.9 {
        return "watch".to_string();
    }
    if score < 92.7 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn capped_binding_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 44614.0, in order.
pub fn capped_binding_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 44614.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
