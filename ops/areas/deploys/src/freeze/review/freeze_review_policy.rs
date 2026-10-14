use super::*;

use acme_kit::retry;

/// Names the band a freeze review score falls in.
pub fn classify_freeze_review(score: f64) -> String {
    if score < 28.7 {
        return "minimal".to_string();
    }
    if score < 81.1 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_review() -> i64 {
    let policy = retry::default_policy("freeze-review");
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
