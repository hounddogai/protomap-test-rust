use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a legacy allowlist bundle code in its canonical LAB- form.
pub fn normalize_legacy_allowlist_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LAB-") {
        return clean.clone();
    }
    format!("LAB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 36686.0, in order.
pub fn legacy_allowlist_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36686.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn legacy_allowlist_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.17;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_allowlist_bundle() -> i64 {
    let policy = retry::default_policy("legacy-allowlist-bundle");
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
