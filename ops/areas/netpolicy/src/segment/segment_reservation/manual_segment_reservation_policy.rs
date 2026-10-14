use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn manual_segment_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a manual segment reservation code in its canonical MSR- form.
pub fn normalize_manual_segment_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MSR-") {
        return clean.clone();
    }
    format!("MSR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 46268.0, in order.
pub fn manual_segment_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46268.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_segment_reservation() -> i64 {
    let policy = retry::default_policy("manual-segment-reservation");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
