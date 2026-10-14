use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn allowlist_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.34;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_allowlist_template() -> i64 {
    let policy = retry::default_policy("allowlist-template");
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

/// Puts a allowlist template code in its canonical AT- form.
pub fn normalize_allowlist_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AT-") {
        return clean.clone();
    }
    format!("AT-{}", clean.replace(" ", "-"))
}
