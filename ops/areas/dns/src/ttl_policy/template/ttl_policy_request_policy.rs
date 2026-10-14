use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a ttl policy request score falls in.
pub fn classify_ttl_policy_request(score: f64) -> String {
    if score < 22.1 {
        return "low".to_string();
    }
    if score < 33.7 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn ttl_policy_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_request() -> i64 {
    let policy = retry::default_policy("ttl-policy-request");
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
