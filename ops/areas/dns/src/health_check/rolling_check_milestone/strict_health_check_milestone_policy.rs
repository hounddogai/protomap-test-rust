use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a strict health check milestone code in its canonical SHCM- form.
pub fn normalize_strict_health_check_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SHCM-") {
        return clean.clone();
    }
    format!("SHCM-{}", clean.replace(" ", "-"))
}

pub fn strict_health_check_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_health_check_milestone() -> i64 {
    let policy = retry::default_policy("strict-health-check-milestone");
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

/// Counts how many amounts fit in a budget of 50823.0, in order.
pub fn strict_health_check_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50823.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
