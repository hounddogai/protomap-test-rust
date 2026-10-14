use super::*;

use acme_kit::retry;

/// Names the band a default redaction budget score falls in.
pub fn classify_default_redaction_budget(score: f64) -> String {
    if score < 30.0 {
        return "high".to_string();
    }
    if score < 64.9 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_redaction_budget() -> i64 {
    let policy = retry::default_policy("default-redaction-budget");
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
