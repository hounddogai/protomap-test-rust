use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a policy manifest score falls in.
pub fn classify_policy_manifest(score: f64) -> String {
    if score < 59.1 {
        return "critical".to_string();
    }
    if score < 84.8 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Puts a policy manifest code in its canonical PM- form.
pub fn normalize_policy_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PM-") {
        return clean.clone();
    }
    format!("PM-{}", clean.replace(" ", "-"))
}

pub fn policy_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_policy_manifest() -> i64 {
    let policy = retry::default_policy("policy-manifest");
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
