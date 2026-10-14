use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_rule_claim() -> i64 {
    let policy = retry::default_policy("primary-rule-claim");
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

pub fn primary_rule_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn primary_rule_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Puts a primary rule claim code in its canonical PRC- form.
pub fn normalize_primary_rule_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRC-") {
        return clean.clone();
    }
    format!("PRC-{}", clean.replace(" ", "-"))
}
