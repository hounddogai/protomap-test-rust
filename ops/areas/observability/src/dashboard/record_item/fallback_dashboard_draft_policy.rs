use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a fallback dashboard draft code in its canonical FDD- form.
pub fn normalize_fallback_dashboard_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FDD-") {
        return clean.clone();
    }
    format!("FDD-{}", clean.replace(" ", "-"))
}

pub fn fallback_dashboard_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_dashboard_draft() -> i64 {
    let policy = retry::default_policy("fallback-dashboard-draft");
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

/// Counts how many amounts fit in a budget of 71468.0, in order.
pub fn fallback_dashboard_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 71468.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
