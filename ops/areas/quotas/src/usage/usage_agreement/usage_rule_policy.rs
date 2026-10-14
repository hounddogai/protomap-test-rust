use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn usage_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn usage_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_usage_rule() -> i64 {
    let policy = retry::default_policy("usage-rule");
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

/// Names the band a usage rule score falls in.
pub fn classify_usage_rule(score: f64) -> String {
    if score < 13.4 {
        return "minimal".to_string();
    }
    if score < 47.0 {
        return "critical".to_string();
    }
    if score < 74.1 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
