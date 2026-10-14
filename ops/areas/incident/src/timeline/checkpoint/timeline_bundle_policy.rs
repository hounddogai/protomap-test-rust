use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn timeline_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a timeline bundle code in its canonical TB- form.
pub fn normalize_timeline_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TB-") {
        return clean.clone();
    }
    format!("TB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_timeline_bundle() -> i64 {
    let policy = retry::default_policy("timeline-bundle");
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

/// Counts how many amounts fit in a budget of 72458.0, in order.
pub fn timeline_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72458.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
