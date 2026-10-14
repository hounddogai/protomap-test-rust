use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_zone_mapping() -> i64 {
    let policy = retry::default_policy("zone-mapping");
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

/// Puts a zone mapping code in its canonical ZM- form.
pub fn normalize_zone_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZM-") {
        return clean.clone();
    }
    format!("ZM-{}", clean.replace(" ", "-"))
}

pub fn zone_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.83;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
