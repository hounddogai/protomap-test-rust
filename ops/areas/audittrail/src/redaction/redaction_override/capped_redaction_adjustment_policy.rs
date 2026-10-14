use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_redaction_adjustment() -> i64 {
    let policy = retry::default_policy("capped-redaction-adjustment");
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

/// Names the band a capped redaction adjustment score falls in.
pub fn classify_capped_redaction_adjustment(score: f64) -> String {
    if score < 15.5 {
        return "low".to_string();
    }
    if score < 42.0 {
        return "minimal".to_string();
    }
    if score < 49.1 {
        return "elevated".to_string();
    }
    "high".to_string()
}
