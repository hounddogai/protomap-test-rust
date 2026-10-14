use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn dns_record_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dns_record_agreement() -> i64 {
    let policy = retry::default_policy("dns-record-agreement");
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

/// Names the band a dns record agreement score falls in.
pub fn classify_dns_record_agreement(score: f64) -> String {
    if score < 53.9 {
        return "elevated".to_string();
    }
    if score < 70.6 {
        return "high".to_string();
    }
    if score < 74.8 {
        return "minimal".to_string();
    }
    "low".to_string()
}
