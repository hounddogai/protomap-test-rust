use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a budget burn reservation code in its canonical BBR- form.
pub fn normalize_budget_burn_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBR-") {
        return clean.clone();
    }
    format!("BBR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_budget_burn_reservation() -> i64 {
    let policy = retry::default_policy("budget-burn-reservation");
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

/// Names the band a budget burn reservation score falls in.
pub fn classify_budget_burn_reservation(score: f64) -> String {
    if score < 2.3 {
        return "elevated".to_string();
    }
    if score < 29.2 {
        return "minimal".to_string();
    }
    if score < 41.1 {
        return "watch".to_string();
    }
    if score < 49.1 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn budget_burn_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
