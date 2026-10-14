use super::*;

use acme_kit::retry;

/// Names the band a approval profile score falls in.
pub fn classify_approval_profile(score: f64) -> String {
    if score < 3.2 {
        return "critical".to_string();
    }
    if score < 39.5 {
        return "high".to_string();
    }
    if score < 48.5 {
        return "minimal".to_string();
    }
    if score < 86.2 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 116.0, in order.
pub fn approval_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 116.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_approval_profile() -> i64 {
    let policy = retry::default_policy("approval-profile");
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
