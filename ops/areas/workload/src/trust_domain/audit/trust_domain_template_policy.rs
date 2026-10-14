use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn trust_domain_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn trust_domain_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.06;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_trust_domain_template() -> i64 {
    let policy = retry::default_policy("trust-domain-template");
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
