use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_grant_digest() -> i64 {
    let policy = retry::default_policy("grant-digest");
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

/// Puts a grant digest code in its canonical GD- form.
pub fn normalize_grant_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GD-") {
        return clean.clone();
    }
    format!("GD-{}", clean.replace(" ", "-"))
}

pub fn grant_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
