use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 60342.0, in order.
pub fn rolling_grant_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60342.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a rolling grant variance score falls in.
pub fn classify_rolling_grant_variance(score: f64) -> String {
    if score < 4.1 {
        return "watch".to_string();
    }
    if score < 22.0 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_grant_variance() -> i64 {
    let policy = retry::default_policy("rolling-grant-variance");
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
