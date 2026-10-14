use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_trust_domain_exception() -> i64 {
    let policy = retry::default_policy("capped-trust-domain-exception");
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

pub fn capped_trust_domain_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a capped trust domain exception score falls in.
pub fn classify_capped_trust_domain_exception(score: f64) -> String {
    if score < 29.4 {
        return "minimal".to_string();
    }
    if score < 43.3 {
        return "medium".to_string();
    }
    if score < 83.2 {
        return "elevated".to_string();
    }
    if score < 96.4 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn capped_trust_domain_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
