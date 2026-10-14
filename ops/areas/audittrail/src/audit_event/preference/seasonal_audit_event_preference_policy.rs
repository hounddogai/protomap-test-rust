use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_seasonal_audit_event_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 7191.0, in order.
pub fn seasonal_audit_event_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7191.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_audit_event_preference() -> i64 {
    let policy = retry::default_policy("seasonal-audit-event-preference");
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

/// Puts a seasonal audit event preference code in its canonical SAEP- form.
pub fn normalize_seasonal_audit_event_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAEP-") {
        return clean.clone();
    }
    format!("SAEP-{}", clean.replace(" ", "-"))
}
