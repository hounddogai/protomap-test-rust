use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_ttl_policy_quota() -> i64 {
    let policy = retry::default_policy("ttl-policy-quota");
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

/// Names the band a ttl policy quota score falls in.
pub fn classify_ttl_policy_quota(score: f64) -> String {
    if score < 2.6 {
        return "ok".to_string();
    }
    if score < 23.1 {
        return "watch".to_string();
    }
    if score < 86.0 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn ttl_policy_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a ttl policy quota code in its canonical TPQ- form.
pub fn normalize_ttl_policy_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPQ-") {
        return clean.clone();
    }
    format!("TPQ-{}", clean.replace(" ", "-"))
}
