use super::*;

use acme_kit::retry;

/// Names the band a federation rule score falls in.
pub fn classify_federation_rule(score: f64) -> String {
    if score < 2.7 {
        return "high".to_string();
    }
    if score < 31.2 {
        return "low".to_string();
    }
    if score < 37.5 {
        return "minimal".to_string();
    }
    if score < 99.7 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_federation_rule() -> i64 {
    let policy = retry::default_policy("federation-rule");
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
