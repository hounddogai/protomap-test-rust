use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a fallback audit event item code in its canonical FAEI- form.
pub fn normalize_fallback_audit_event_item_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FAEI-") {
        return clean.clone();
    }
    format!("FAEI-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_audit_event_item() -> i64 {
    let policy = retry::default_policy("fallback-audit-event-item");
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

pub fn fallback_audit_event_item_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 28173.0, in order.
pub fn fallback_audit_event_item_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28173.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
