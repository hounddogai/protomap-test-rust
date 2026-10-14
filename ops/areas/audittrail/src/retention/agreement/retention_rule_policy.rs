use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_retention_rule() -> i64 {
    let policy = retry::default_policy("retention-rule");
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

/// Names the band a retention rule score falls in.
pub fn classify_retention_rule(score: f64) -> String {
    if score < 28.3 {
        return "high".to_string();
    }
    if score < 33.4 {
        return "watch".to_string();
    }
    if score < 60.8 {
        return "medium".to_string();
    }
    if score < 90.0 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
