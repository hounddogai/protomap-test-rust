use super::*;

use acme_kit::retry;

/// Names the band a primary trust domain override score falls in.
pub fn classify_primary_trust_domain_override(score: f64) -> String {
    if score < 3.6 {
        return "medium".to_string();
    }
    if score < 32.4 {
        return "low".to_string();
    }
    if score < 33.4 {
        return "minimal".to_string();
    }
    if score < 82.1 {
        return "watch".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_trust_domain_override() -> i64 {
    let policy = retry::default_policy("primary-trust-domain-override");
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
