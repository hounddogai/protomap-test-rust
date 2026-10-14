use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn weighted_region_pin_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.12;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a weighted region pin preference code in its canonical WRPP- form.
pub fn normalize_weighted_region_pin_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WRPP-") {
        return clean.clone();
    }
    format!("WRPP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_region_pin_preference() -> i64 {
    let policy = retry::default_policy("weighted-region-pin-preference");
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
