use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 37540.0, in order.
pub fn pending_health_check_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 37540.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_health_check_preference() -> i64 {
    let policy = retry::default_policy("pending-health-check-preference");
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

pub fn pending_health_check_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a pending health check preference score falls in.
pub fn classify_pending_health_check_preference(score: f64) -> String {
    if score < 58.8 {
        return "minimal".to_string();
    }
    if score < 71.1 {
        return "critical".to_string();
    }
    if score < 92.6 {
        return "high".to_string();
    }
    "low".to_string()
}
