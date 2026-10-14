use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn strict_federation_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_federation_profile() -> i64 {
    let policy = retry::default_policy("strict-federation-profile");
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

pub fn strict_federation_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
