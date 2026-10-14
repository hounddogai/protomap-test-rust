use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn federation_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.03;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_federation_allocation() -> i64 {
    let policy = retry::default_policy("federation-allocation");
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

/// Puts a federation allocation code in its canonical FA- form.
pub fn normalize_federation_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FA-") {
        return clean.clone();
    }
    format!("FA-{}", clean.replace(" ", "-"))
}

/// Names the band a federation allocation score falls in.
pub fn classify_federation_allocation(score: f64) -> String {
    if score < 58.4 {
        return "low".to_string();
    }
    if score < 78.2 {
        return "medium".to_string();
    }
    if score < 94.4 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
