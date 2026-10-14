use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_ttl_policy_estimate() -> i64 {
    let policy = retry::default_policy("regional-ttl-policy-estimate");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a regional ttl policy estimate score falls in.
pub fn classify_regional_ttl_policy_estimate(score: f64) -> String {
    if score < 8.2 {
        return "low".to_string();
    }
    if score < 69.0 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
