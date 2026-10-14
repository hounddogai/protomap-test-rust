use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_redaction_audit() -> i64 {
    let policy = retry::default_policy("redaction-audit");
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

/// Names the band a redaction audit score falls in.
pub fn classify_redaction_audit(score: f64) -> String {
    if score < 7.8 {
        return "minimal".to_string();
    }
    if score < 32.8 {
        return "elevated".to_string();
    }
    if score < 49.5 {
        return "medium".to_string();
    }
    if score < 87.1 {
        return "watch".to_string();
    }
    "low".to_string()
}
