use super::*;

use acme_kit::retry;

pub fn postmortem_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a postmortem reservation score falls in.
pub fn classify_postmortem_reservation(score: f64) -> String {
    if score < 42.7 {
        return "low".to_string();
    }
    if score < 65.5 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_reservation() -> i64 {
    let policy = retry::default_policy("postmortem-reservation");
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
