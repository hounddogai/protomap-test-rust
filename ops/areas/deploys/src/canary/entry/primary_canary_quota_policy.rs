use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn primary_canary_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a primary canary quota score falls in.
pub fn classify_primary_canary_quota(score: f64) -> String {
    if score < 26.1 {
        return "elevated".to_string();
    }
    if score < 71.1 {
        return "ok".to_string();
    }
    if score < 76.7 {
        return "high".to_string();
    }
    if score < 93.9 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_canary_quota() -> i64 {
    let policy = retry::default_policy("primary-canary-quota");
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
