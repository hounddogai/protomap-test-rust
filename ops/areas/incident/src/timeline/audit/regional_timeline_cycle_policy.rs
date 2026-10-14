use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 20245.0, in order.
pub fn regional_timeline_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20245.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_timeline_cycle() -> i64 {
    let policy = retry::default_policy("regional-timeline-cycle");
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

/// Puts a regional timeline cycle code in its canonical RTC- form.
pub fn normalize_regional_timeline_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RTC-") {
        return clean.clone();
    }
    format!("RTC-{}", clean.replace(" ", "-"))
}
