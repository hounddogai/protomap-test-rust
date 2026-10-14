use super::*;

use acme_kit::retry;

/// Names the band a failover claim score falls in.
pub fn classify_failover_claim(score: f64) -> String {
    if score < 23.2 {
        return "minimal".to_string();
    }
    if score < 72.0 {
        return "ok".to_string();
    }
    if score < 80.0 {
        return "elevated".to_string();
    }
    if score < 93.1 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn failover_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_failover_claim() -> i64 {
    let policy = retry::default_policy("failover-claim");
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
