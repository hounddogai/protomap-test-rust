use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a trust domain target score falls in.
pub fn classify_trust_domain_target(score: f64) -> String {
    if score < 4.0 {
        return "low".to_string();
    }
    if score < 42.9 {
        return "medium".to_string();
    }
    if score < 60.8 {
        return "ok".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_trust_domain_target() -> i64 {
    let policy = retry::default_policy("trust-domain-target");
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

pub fn trust_domain_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.41;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn trust_domain_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
