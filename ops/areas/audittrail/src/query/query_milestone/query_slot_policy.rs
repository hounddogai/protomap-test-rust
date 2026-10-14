use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn query_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_query_slot() -> i64 {
    let policy = retry::default_policy("query-slot");
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

/// Puts a query slot code in its canonical QS- form.
pub fn normalize_query_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QS-") {
        return clean.clone();
    }
    format!("QS-{}", clean.replace(" ", "-"))
}
