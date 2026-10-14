use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn allowlist_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn allowlist_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_allowlist_claim() -> i64 {
    let policy = retry::default_policy("allowlist-claim");
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

/// Names the band a allowlist claim score falls in.
pub fn classify_allowlist_claim(score: f64) -> String {
    if score < 10.9 {
        return "ok".to_string();
    }
    if score < 21.9 {
        return "medium".to_string();
    }
    if score < 61.1 {
        return "elevated".to_string();
    }
    if score < 86.8 {
        return "high".to_string();
    }
    "watch".to_string()
}
