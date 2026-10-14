use super::*;

use acme_kit::retry;

/// Names the band a capped release segment score falls in.
pub fn classify_capped_release_segment(score: f64) -> String {
    if score < 11.3 {
        return "low".to_string();
    }
    if score < 19.0 {
        return "high".to_string();
    }
    if score < 81.0 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_release_segment() -> i64 {
    let policy = retry::default_policy("capped-release-segment");
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
