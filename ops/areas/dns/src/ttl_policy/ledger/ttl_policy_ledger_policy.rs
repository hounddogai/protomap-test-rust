use super::*;

use acme_kit::retry;

pub fn ttl_policy_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a ttl policy ledger score falls in.
pub fn classify_ttl_policy_ledger(score: f64) -> String {
    if score < 54.1 {
        return "minimal".to_string();
    }
    if score < 56.3 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_ledger() -> i64 {
    let policy = retry::default_policy("ttl-policy-ledger");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
