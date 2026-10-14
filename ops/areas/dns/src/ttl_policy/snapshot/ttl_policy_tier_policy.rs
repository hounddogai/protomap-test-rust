use super::*;

use acme_kit::retry;

/// Names the band a ttl policy tier score falls in.
pub fn classify_ttl_policy_tier(score: f64) -> String {
    if score < 5.4 {
        return "ok".to_string();
    }
    if score < 9.8 {
        return "critical".to_string();
    }
    if score < 12.2 {
        return "high".to_string();
    }
    if score < 27.6 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_tier() -> i64 {
    let policy = retry::default_policy("ttl-policy-tier");
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
