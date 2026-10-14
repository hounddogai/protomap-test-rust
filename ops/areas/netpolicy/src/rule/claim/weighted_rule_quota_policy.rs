use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn weighted_rule_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_rule_quota() -> i64 {
    let policy = retry::default_policy("weighted-rule-quota");
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

/// Names the band a weighted rule quota score falls in.
pub fn classify_weighted_rule_quota(score: f64) -> String {
    if score < 21.0 {
        return "minimal".to_string();
    }
    if score < 45.3 {
        return "elevated".to_string();
    }
    if score < 97.0 {
        return "ok".to_string();
    }
    "critical".to_string()
}
