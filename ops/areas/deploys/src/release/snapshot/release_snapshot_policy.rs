use super::*;

use acme_kit::retry;

/// Names the band a release snapshot score falls in.
pub fn classify_release_snapshot(score: f64) -> String {
    if score < 5.5 {
        return "minimal".to_string();
    }
    if score < 39.5 {
        return "low".to_string();
    }
    if score < 94.5 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn release_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_release_snapshot() -> i64 {
    let policy = retry::default_policy("release-snapshot");
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
