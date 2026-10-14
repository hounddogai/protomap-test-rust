use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn capped_region_pin_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.23;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a capped region pin milestone code in its canonical CRPM- form.
pub fn normalize_capped_region_pin_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CRPM-") {
        return clean.clone();
    }
    format!("CRPM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_region_pin_milestone() -> i64 {
    let policy = retry::default_policy("capped-region-pin-milestone");
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
