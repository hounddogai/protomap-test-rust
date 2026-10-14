use super::*;

use acme_kit::retry;

pub fn primary_ttl_policy_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_ttl_policy_profile() -> i64 {
    let policy = retry::default_policy("primary-ttl-policy-profile");
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

/// Names the band a primary ttl policy profile score falls in.
pub fn classify_primary_ttl_policy_profile(score: f64) -> String {
    if score < 16.3 {
        return "high".to_string();
    }
    if score < 31.6 {
        return "elevated".to_string();
    }
    if score < 53.5 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
