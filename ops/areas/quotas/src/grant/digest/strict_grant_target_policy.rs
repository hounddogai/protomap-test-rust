use super::*;

use acme_kit::retry;

pub fn strict_grant_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a strict grant target score falls in.
pub fn classify_strict_grant_target(score: f64) -> String {
    if score < 27.9 {
        return "low".to_string();
    }
    if score < 30.6 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_grant_target() -> i64 {
    let policy = retry::default_policy("strict-grant-target");
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
