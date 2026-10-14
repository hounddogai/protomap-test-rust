use super::*;

use acme_kit::retry;

pub fn query_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_query_milestone() -> i64 {
    let policy = retry::default_policy("query-milestone");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
