use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn lenient_delegation_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.12;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a lenient delegation profile code in its canonical LDP- form.
pub fn normalize_lenient_delegation_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LDP-") {
        return clean.clone();
    }
    format!("LDP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_delegation_profile() -> i64 {
    let policy = retry::default_policy("lenient-delegation-profile");
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
